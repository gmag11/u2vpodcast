use chrono::{
    DateTime,
    Utc,
};
use std::{
    time::UNIX_EPOCH,
    path::{
        Component,
        Path,
        PathBuf,
    },
};

use actix_web::{
    HttpRequest,
    HttpResponse,
    http::{
        StatusCode,
        Method,
        header,
    },
    web::Path as WebPath,
};
use tokio::io::{
    AsyncReadExt,
    AsyncSeekExt,
    SeekFrom,
};
use tokio_util::io::ReaderStream;
use tracing::{
    info,
    debug,
};

use crate::models::audios_dir;

/// Resolves a `/media/{path:.*}` segment under the audios directory, rejecting
/// any path traversal component.
fn resolve_media(relative: &str) -> Option<PathBuf> {
    let rel = Path::new(relative);
    if rel
        .components()
        .any(|c| matches!(c, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    {
        return None;
    }
    Some(Path::new(audios_dir()).join(rel))
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
        meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)
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

pub async fn serve_media(req: HttpRequest, path: WebPath<String>) -> HttpResponse {
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
        debug!("media 206 {} {} bytes {start}-{end}/{total}", req.method(), relative);
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