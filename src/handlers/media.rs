use chrono::{DateTime, Utc};
use std::{
    path::{Component, Path, PathBuf},
    time::UNIX_EPOCH,
};

use actix_web::{
    http::{header, Method, StatusCode},
    web::{Data, Path as WebPath},
    HttpRequest, HttpResponse,
};
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use tokio_util::io::ReaderStream;
use tracing::{debug, info};

use crate::models::{audios_dir, AppState, Episode};

/// Resolves a `/media/{path:.*}` segment under the audios directory, rejecting
/// any path traversal component.
fn resolve_media(relative: &str) -> Option<PathBuf> {
    let rel = Path::new(relative);
    if rel.components().any(|c| {
        matches!(
            c,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return None;
    }
    Some(Path::new(audios_dir()).join(rel))
}

/// Returns the `yt_id` when `filename` is the plain `{yt_id}.mp3` form used by
/// the stable feed enclosure URL. Hash-versioned derivatives
/// (`{yt_id}.sponsorblock.{hash}.mp3`) and the `.original.mp3` alias are
/// excluded so they keep resolving directly.
fn stable_media_yt_id(filename: &str) -> Option<&str> {
    let stem = filename.strip_suffix(".mp3")?;
    if stem.contains(".sponsorblock.") || stem.ends_with(".original") {
        return None;
    }
    Some(stem)
}

/// Maps a `{yt_id}.original.mp3` request to the on-disk original `{yt_id}.mp3`.
fn original_media_filename(filename: &str) -> Option<String> {
    let yt_id = filename.strip_suffix(".original.mp3")?;
    Some(format!("{yt_id}.mp3"))
}

fn mime_for(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "mp3" => "audio/mpeg",
        "ogg" | "oga" | "opus" => "audio/ogg",
        "m4a" => "audio/mp4",
        "mp4" => "audio/mp4",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        _ => "application/octet-stream",
    }
}

/// Parses the first single byte-range of a `Range: bytes=...` header into an
/// inclusive `(start, end)` within `[0, total)`. Supports `start-end`,
/// `start-` (open-ended) and `-suffix`. Returns `None` when unsatisfiable.
/// The open-ended form is served as `206` too: actix-files answered it with a
/// bare `200`, which makes browsers treat the resource as range-unsupported and
/// limit `seekable` to the downloaded prefix (media-range-handling).
fn parse_range(raw: &str, total: u64) -> Option<(u64, u64)> {
    if total == 0 {
        return None;
    }
    let spec = raw.trim().strip_prefix("bytes=")?.split(',').next()?.trim();
    if let Some(suffix) = spec.strip_prefix('-') {
        let n: u64 = suffix.trim().parse().ok()?;
        if n == 0 {
            return None;
        }
        let start = total.saturating_sub(n);
        return Some((start, total - 1));
    }
    let (start_s, rest) = spec.split_once('-')?;
    let start: u64 = start_s.trim().parse().ok()?;
    if start >= total {
        return None;
    }
    let end_s = rest.trim();
    let end: u64 = if end_s.is_empty() {
        total - 1
    } else {
        end_s.parse::<u64>().ok()?.min(total - 1)
    };
    if start > end {
        return None;
    }
    Some((start, end))
}

/// Maps a file's metadata to strong validators so the browser's media cache
/// can revalidate (Chrome otherwise serves stale truncated prefixes, which
/// freezes `seekable` and clamps resume seeks).
fn validators(meta: &std::fs::Metadata) -> (String, String) {
    let etag = format!(
        "\"{:x}-{:x}\"",
        meta.len(),
        meta.modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );
    let last_modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| {
            DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH + d)
                .format("%a, %d %b %Y %H:%M:%S GMT")
                .to_string()
        })
        .unwrap_or_else(|| "Thu, 01 Jan 1970 00:00:00 GMT".to_string());
    (etag, last_modified)
}

/// Resolves the requested relative media path to the physical file to stream.
/// `.original.mp3` maps to the original file; the plain `{yt_id}.mp3` form maps
/// to the active SponsorBlock derivative when enabled and present. Hash-versioned
/// derivative names and every other path resolve directly.
async fn resolve_selected_media(relative: &str, full: PathBuf, data: &AppState) -> PathBuf {
    let Some((parent, filename)) = relative.rsplit_once('/') else {
        return full;
    };
    if let Some(yt_id) = stable_media_yt_id(filename) {
        if data.config.sponsorblock_enabled {
            if let Ok(Some(processed)) =
                Episode::active_processed_filename(&data.pool, parent, yt_id).await
            {
                let candidate = full.with_file_name(&processed);
                if tokio::fs::metadata(&candidate)
                    .await
                    .map(|metadata| metadata.is_file())
                    .unwrap_or(false)
                {
                    debug!("media stable {} -> {}", relative, processed);
                    return candidate;
                }
            }
        }
    } else if let Some(original) = original_media_filename(filename) {
        return full.with_file_name(original);
    }
    full
}

pub async fn serve_media(
    req: HttpRequest,
    path: WebPath<String>,
    data: Data<AppState>,
) -> HttpResponse {
    let relative = path.into_inner();
    let range_hdr = req
        .headers()
        .get(header::RANGE)
        .map(|v| v.to_str().unwrap_or("?").to_string());
    info!(
        "media {} {} range={:?} ims={:?}",
        req.method(),
        relative,
        range_hdr,
        req.headers()
            .get(header::IF_MODIFIED_SINCE)
            .map(|v| v.to_str().unwrap_or("?")),
    );
    let Some(full) = resolve_media(&relative) else {
        debug!("media 404 {} {}", req.method(), relative);
        return HttpResponse::NotFound().finish();
    };
    let full = resolve_selected_media(&relative, full, &data).await;
    let Ok(meta) = tokio::fs::metadata(&full).await else {
        debug!("media 404 {} {}", req.method(), relative);
        return HttpResponse::NotFound().finish();
    };
    if !meta.is_file() {
        debug!("media 404 {} {}", req.method(), relative);
        return HttpResponse::NotFound().finish();
    }
    let (etag, last_modified) = validators(&meta);
    let total = meta.len();
    let mime = mime_for(&full);
    let is_head = req.method() == Method::HEAD;

    let range = range_hdr.as_deref().and_then(|v| parse_range(v, total));

    // Satisfiable single range (open-ended included) → always 206.
    if let Some((start, end)) = range {
        let length = end - start + 1;
        let mut builder = HttpResponse::build(StatusCode::PARTIAL_CONTENT);
        builder.insert_header((header::ACCEPT_RANGES, "bytes"));
        builder.insert_header((
            header::CONTENT_RANGE,
            format!("bytes {start}-{end}/{total}"),
        ));
        builder.insert_header((header::CONTENT_TYPE, mime));
        builder.insert_header((header::CONTENT_LENGTH, length));
        builder.insert_header((header::LAST_MODIFIED, last_modified.clone()));
        builder.insert_header((header::ETAG, etag.clone()));
        if is_head {
            debug!("media 206 HEAD {} {}", req.method(), relative);
            return builder.finish();
        }
        let Ok(mut file) = tokio::fs::File::open(&full).await else {
            debug!("media 404 open {}", relative);
            return HttpResponse::NotFound().finish();
        };
        if file.seek(SeekFrom::Start(start)).await.is_err() {
            return HttpResponse::InternalServerError().finish();
        }
        debug!(
            "media 206 {} {} bytes {start}-{end}/{total}",
            req.method(),
            relative
        );
        return builder.streaming(ReaderStream::new(file.take(length)));
    }

    // A malformed/unsatisfiable Range header is still answered 416 when the
    // header is present but not satisfiable.
    if range_hdr.is_some() {
        debug!("media 416 {} {}", req.method(), relative);
        return HttpResponse::build(StatusCode::RANGE_NOT_SATISFIABLE)
            .insert_header((header::CONTENT_RANGE, format!("bytes */{total}")))
            .finish();
    }

    // Conditional GET: let the media cache revalidate instead of re-serving a
    // stale prefix (304 carries no body).
    {
        let ims = req
            .headers()
            .get(header::IF_MODIFIED_SINCE)
            .and_then(|v| v.to_str().ok());
        if !is_head && ims == Some(last_modified.as_str()) {
            debug!("media 304 {} {}", req.method(), relative);
            let mut resp = HttpResponse::build(StatusCode::NOT_MODIFIED);
            resp.insert_header((header::LAST_MODIFIED, last_modified));
            resp.insert_header((header::ETAG, etag));
            return resp.finish();
        }
    }

    // No Range header → the full file.
    let mut builder = HttpResponse::Ok();
    builder.insert_header((header::ACCEPT_RANGES, "bytes"));
    builder.insert_header((header::CONTENT_TYPE, mime));
    builder.insert_header((header::CONTENT_LENGTH, total));
    builder.insert_header((header::LAST_MODIFIED, last_modified));
    builder.insert_header((header::ETAG, etag));
    if is_head {
        debug!("media 200 HEAD {}", relative);
        return builder.finish();
    }
    let Ok(file) = tokio::fs::File::open(&full).await else {
        debug!("media 404 open {}", relative);
        return HttpResponse::NotFound().finish();
    };
    debug!("media 200 {} {total} bytes", req.method());
    builder.streaming(ReaderStream::new(file))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{config::test_config, SponsorBlockCache};
    use actix_web::{body::to_bytes, test::TestRequest};
    use chrono::Utc;
    use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions};

    async fn test_state(sponsorblock_enabled: bool) -> Data<AppState> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("memory pool");
        Migrator::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations"))
            .await
            .expect("load migrations")
            .run(&pool)
            .await
            .expect("run migrations");
        let mut config = test_config();
        config.sponsorblock_enabled = sponsorblock_enabled;
        Data::new(AppState { config, pool })
    }

    async fn add_episode(
        state: &Data<AppState>,
        slug: &str,
        yt_id: &str,
        processed: Option<&str>,
    ) {
        let now = Utc::now();
        let channel_id: i64 = sqlx::query_scalar(
            "INSERT INTO channels (url, title, slug, active, description, image, first, max, created_at, updated_at) \
             VALUES ('https://example.com', 'Channel', $1, TRUE, '', '', $2, 5, $2, $2) RETURNING id",
        )
        .bind(slug)
        .bind(now)
        .fetch_one(&state.pool)
        .await
        .expect("insert channel");
        let episode_id: i64 = sqlx::query_scalar(
            "INSERT INTO episodes (channel_id, title, yt_id, webpage_url, published_at, duration, created_at, updated_at) \
             VALUES ($1, 'Episode', $2, 'https://example.com/v', $3, '00:01:00', $3, $3) RETURNING id",
        )
        .bind(channel_id)
        .bind(yt_id)
        .bind(now)
        .fetch_one(&state.pool)
        .await
        .expect("insert episode");
        if let Some(filename) = processed {
            SponsorBlockCache::upsert_success(
                &state.pool,
                episode_id,
                &[],
                "snapshot",
                "processing",
                Some(filename),
                Some(50.0),
            )
            .await
            .expect("store sponsorblock cache");
        }
    }

    fn fixture_dir(slug: &str) -> PathBuf {
        let directory = Path::new(audios_dir()).join(slug);
        std::fs::create_dir_all(&directory).expect("create fixture directory");
        directory
    }

    async fn get_media(state: &Data<AppState>, relative: &str) -> HttpResponse {
        serve_media(
            TestRequest::get()
                .uri(&format!("/media/{relative}"))
                .to_http_request(),
            WebPath::from(relative.to_string()),
            state.clone(),
        )
        .await
    }

    #[actix_web::test]
    async fn serves_hash_versioned_mp3_with_full_head_range_and_conditional_requests() {
        let state = test_state(true).await;
        let slug = format!("sponsorblock_media_test_{}", rand::random::<u64>());
        let relative = format!("{slug}/video.sponsorblock.abcdef0123456789.mp3");
        let directory = fixture_dir(&slug);
        std::fs::write(
            directory.join("video.sponsorblock.abcdef0123456789.mp3"),
            b"0123456789",
        )
        .unwrap();

        let get = serve_media(
            TestRequest::get()
                .uri(&format!("/media/{relative}"))
                .to_http_request(),
            WebPath::from(relative.clone()),
            state.clone(),
        )
        .await;
        assert_eq!(get.status(), StatusCode::OK);
        let modified = get
            .headers()
            .get(header::LAST_MODIFIED)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert_eq!(
            to_bytes(get.into_body()).await.unwrap(),
            b"0123456789".as_slice()
        );

        let head = serve_media(
            TestRequest::default()
                .method(Method::HEAD)
                .to_http_request(),
            WebPath::from(relative.clone()),
            state.clone(),
        )
        .await;
        assert_eq!(head.status(), StatusCode::OK);
        assert_eq!(head.headers().get(header::CONTENT_LENGTH).unwrap(), "10");

        let range = serve_media(
            TestRequest::get()
                .insert_header((header::RANGE, "bytes=2-5"))
                .to_http_request(),
            WebPath::from(relative.clone()),
            state.clone(),
        )
        .await;
        assert_eq!(range.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(
            range.headers().get(header::CONTENT_RANGE).unwrap(),
            "bytes 2-5/10"
        );
        assert_eq!(
            to_bytes(range.into_body()).await.unwrap(),
            b"2345".as_slice()
        );

        let conditional = serve_media(
            TestRequest::get()
                .insert_header((header::IF_MODIFIED_SINCE, modified))
                .to_http_request(),
            WebPath::from(relative),
            state.clone(),
        )
        .await;
        assert_eq!(conditional.status(), StatusCode::NOT_MODIFIED);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[actix_web::test]
    async fn stable_url_serves_processed_and_original_alias_serves_original() {
        let state = test_state(true).await;
        let slug = format!("stable_media_{}", rand::random::<u64>());
        let directory = fixture_dir(&slug);
        std::fs::write(directory.join("abc123.mp3"), b"original").unwrap();
        std::fs::write(
            directory.join("abc123.sponsorblock.abcdef.mp3"),
            b"processed",
        )
        .unwrap();
        add_episode(
            &state,
            &slug,
            "abc123",
            Some("abc123.sponsorblock.abcdef.mp3"),
        )
        .await;

        let stable = get_media(&state, &format!("{slug}/abc123.mp3")).await;
        assert_eq!(stable.status(), StatusCode::OK);
        assert_eq!(
            to_bytes(stable.into_body()).await.unwrap(),
            b"processed".as_slice()
        );

        let original = get_media(&state, &format!("{slug}/abc123.original.mp3")).await;
        assert_eq!(original.status(), StatusCode::OK);
        assert_eq!(
            to_bytes(original.into_body()).await.unwrap(),
            b"original".as_slice()
        );

        std::fs::remove_dir_all(directory).unwrap();
    }

    #[actix_web::test]
    async fn stable_url_serves_original_when_sponsorblock_is_disabled() {
        let state = test_state(false).await;
        let slug = format!("stable_disabled_{}", rand::random::<u64>());
        let directory = fixture_dir(&slug);
        std::fs::write(directory.join("abc123.mp3"), b"original").unwrap();
        std::fs::write(
            directory.join("abc123.sponsorblock.abcdef.mp3"),
            b"processed",
        )
        .unwrap();
        add_episode(
            &state,
            &slug,
            "abc123",
            Some("abc123.sponsorblock.abcdef.mp3"),
        )
        .await;

        let response = get_media(&state, &format!("{slug}/abc123.mp3")).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            to_bytes(response.into_body()).await.unwrap(),
            b"original".as_slice()
        );

        std::fs::remove_dir_all(directory).unwrap();
    }

    #[actix_web::test]
    async fn stable_url_serves_original_without_a_derivative() {
        let state = test_state(true).await;
        let slug = format!("stable_noderiv_{}", rand::random::<u64>());
        let directory = fixture_dir(&slug);
        std::fs::write(directory.join("abc123.mp3"), b"original").unwrap();
        add_episode(&state, &slug, "abc123", None).await;

        let response = get_media(&state, &format!("{slug}/abc123.mp3")).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            to_bytes(response.into_body()).await.unwrap(),
            b"original".as_slice()
        );

        std::fs::remove_dir_all(directory).unwrap();
    }

    #[actix_web::test]
    async fn stable_url_serves_original_when_the_derivative_is_missing() {
        let state = test_state(true).await;
        let slug = format!("stable_missing_{}", rand::random::<u64>());
        let directory = fixture_dir(&slug);
        std::fs::write(directory.join("abc123.mp3"), b"original").unwrap();
        add_episode(
            &state,
            &slug,
            "abc123",
            Some("abc123.sponsorblock.absent.mp3"),
        )
        .await;

        let response = get_media(&state, &format!("{slug}/abc123.mp3")).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            to_bytes(response.into_body()).await.unwrap(),
            b"original".as_slice()
        );

        std::fs::remove_dir_all(directory).unwrap();
    }

    #[actix_web::test]
    async fn plain_media_file_without_an_episode_is_served() {
        let state = test_state(true).await;
        let slug = format!("stable_orphan_{}", rand::random::<u64>());
        let directory = fixture_dir(&slug);
        std::fs::write(directory.join("orphan.mp3"), b"orphan").unwrap();

        let response = get_media(&state, &format!("{slug}/orphan.mp3")).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            to_bytes(response.into_body()).await.unwrap(),
            b"orphan".as_slice()
        );

        std::fs::remove_dir_all(directory).unwrap();
    }

    #[actix_web::test]
    async fn stable_url_returns_404_when_no_representation_exists() {
        let state = test_state(true).await;
        let slug = format!("stable_404_{}", rand::random::<u64>());
        add_episode(
            &state,
            &slug,
            "abc123",
            Some("abc123.sponsorblock.absent.mp3"),
        )
        .await;

        let response = get_media(&state, &format!("{slug}/abc123.mp3")).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let alias = get_media(&state, &format!("{slug}/abc123.original.mp3")).await;
        assert_eq!(alias.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn media_route_injects_app_state_and_resolves_stable_url() {
        use actix_web::{test, web, App};

        let state = test_state(true).await;
        let slug = format!("stable_route_{}", rand::random::<u64>());
        let directory = fixture_dir(&slug);
        std::fs::write(directory.join("abc123.mp3"), b"original").unwrap();
        std::fs::write(
            directory.join("abc123.sponsorblock.abcdef.mp3"),
            b"processed",
        )
        .unwrap();
        add_episode(
            &state,
            &slug,
            "abc123",
            Some("abc123.sponsorblock.abcdef.mp3"),
        )
        .await;

        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/media/{path:.*}", web::get().to(serve_media)),
        )
        .await;

        let response = test::call_service(
            &app,
            TestRequest::get()
                .uri(&format!("/media/{slug}/abc123.mp3"))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = test::read_body(response).await;
        assert_eq!(body, b"processed".as_slice());

        std::fs::remove_dir_all(directory).unwrap();
    }
}
