use actix_session::Session;
use actix_web::{
    web::{Data, Path},
    HttpRequest, HttpResponse,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::Path as FsPath;
use tracing::{error, info};

use super::{
    super::models::{audios_dir, CResponse, Episode},
    AppState,
};
use crate::utils::share::{mint_token, verify_token};

#[derive(Serialize)]
struct ShareLink {
    url: String,
    expires_at: DateTime<Utc>,
}

/// Authenticated endpoint that mints a public share link for one episode.
/// Registered under the JSON API session guard, so reaching it implies a valid
/// session; the link itself is later redeemed without any credentials.
pub async fn create_share(
    data: Data<AppState>,
    session: Session,
    yt_id: Path<String>,
) -> HttpResponse {
    info!("create_share");
    let yt_id = yt_id.into_inner();
    match Episode::read_by_yt_id_with_channel(&data.pool, &yt_id).await {
        Ok(_) => {
            let (token, expires_at) =
                mint_token(&data.config.secret_key, &yt_id, data.config.share_ttl_days);
            let url = format!("{}/app/share/{}", data.config.url, token);
            CResponse::ok(session, ShareLink { url, expires_at })
        }
        Err(e) => {
            error!("Error minting share link for {yt_id}: {e}");
            CResponse::ko(e.status_code(), session)
        }
    }
}

/// Minimal metadata exposed to an anonymous visitor through a share token.
/// Deliberately narrow: no user, progress, favorite, chapter, or SponsorBlock
/// fields, and no other episode.
#[derive(Serialize)]
struct SharedEpisode {
    title: String,
    channel_title: String,
    description: String,
    image: String,
    duration: String,
    published_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    audio_url: String,
}

/// Public metadata for the share page. Authorized solely by the token; a
/// malformed, forged, or expired token is answered `404` like an unknown path.
pub async fn get_shared_episode(data: Data<AppState>, token: Path<String>) -> HttpResponse {
    info!("get_shared_episode");
    let token = token.into_inner();
    let Some(verified) = verify_token(&data.config.secret_key, &token, Utc::now().timestamp())
    else {
        return HttpResponse::NotFound().finish();
    };
    match Episode::read_by_yt_id_with_channel(&data.pool, &verified.yt_id).await {
        Ok(episode) => HttpResponse::Ok().json(SharedEpisode {
            title: episode.title,
            channel_title: episode.channel_title,
            description: episode.description,
            image: episode.image,
            duration: episode.duration,
            published_at: episode.published_at,
            expires_at: verified.expires_at,
            audio_url: format!("{}/s/{}/audio.mp3", data.config.url, token),
        }),
        Err(e) => {
            info!("shared episode unavailable: {e}");
            HttpResponse::NotFound().finish()
        }
    }
}

/// Public audio for a share token. Serves the episode's current representation
/// (processed when SponsorBlock is enabled and a valid derivative exists,
/// otherwise the original) through the same streaming core as the protected
/// media route, but with `Cache-Control: private` and no credential check.
pub async fn get_shared_audio(
    req: HttpRequest,
    data: Data<AppState>,
    token: Path<String>,
) -> HttpResponse {
    info!("get_shared_audio");
    let Some(verified) = verify_token(
        &data.config.secret_key,
        &token.into_inner(),
        Utc::now().timestamp(),
    ) else {
        return HttpResponse::NotFound().finish();
    };
    let Ok(episode) = Episode::read_by_yt_id_with_channel(&data.pool, &verified.yt_id).await else {
        return HttpResponse::NotFound().finish();
    };
    if episode.channel_slug.is_empty() {
        return HttpResponse::NotFound().finish();
    }
    let channel_dir = FsPath::new(audios_dir()).join(&episode.channel_slug);
    let selected = episode.selected_media(&channel_dir, data.config.sponsorblock_enabled);
    let full = channel_dir.join(&selected.filename);
    super::media::stream_file(&req, &full, Some("private")).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::test_config;
    use crate::models::SponsorBlockCache;
    use crate::utils::middleware::RequireSession;
    use crate::utils::share::{mint_token_with_exp, verify_token};
    use actix_session::{storage::CookieSessionStore, SessionExt, SessionMiddleware};
    use actix_web::{
        body::to_bytes,
        cookie::Key,
        http::{header, Method, StatusCode},
        test, web, App,
    };
    use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions};
    use std::path::Path as FsPath;

    async fn fixture() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        Migrator::new(FsPath::new(env!("CARGO_MANIFEST_DIR")).join("migrations"))
            .await
            .unwrap()
            .run(&pool)
            .await
            .unwrap();
        let now = Utc::now();
        let channel_id: i64 = sqlx::query_scalar(
            "INSERT INTO channels (url, title, slug, active, description, image, first, max, created_at, updated_at) \
             VALUES ('https://example.com', 'Channel', 'channel', TRUE, 'desc', 'img', $1, 5, $1, $1) RETURNING id",
        )
        .bind(now)
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO episodes (channel_id, title, yt_id, webpage_url, published_at, duration, created_at, updated_at) \
             VALUES ($1, 'Episode', 'abc123', 'https://example.com/v', $2, '00:10:00', $2, $2)",
        )
        .bind(channel_id)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    fn state(pool: sqlx::SqlitePool) -> Data<AppState> {
        state_with(pool, test_config())
    }

    fn state_with(pool: sqlx::SqlitePool, config: crate::models::Config) -> Data<AppState> {
        Data::new(AppState { config, pool })
    }

    /// Writes `audios/{slug}/{filename}` and returns the episode directory.
    fn write_media(slug: &str, filename: &str, bytes: &[u8]) -> std::path::PathBuf {
        let directory = std::path::Path::new(audios_dir()).join(slug);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join(filename), bytes).unwrap();
        directory
    }

    async fn set_channel_slug(pool: &sqlx::SqlitePool, slug: &str) {
        sqlx::query("UPDATE channels SET slug = $1")
            .bind(slug)
            .execute(pool)
            .await
            .unwrap();
    }

    fn media_request() -> HttpRequest {
        test::TestRequest::get().to_http_request()
    }

    fn media_request_with_range(range: &str) -> HttpRequest {
        test::TestRequest::get()
            .insert_header((header::RANGE, range))
            .to_http_request()
    }

    fn media_head_request() -> HttpRequest {
        test::TestRequest::default()
            .method(Method::HEAD)
            .to_http_request()
    }

    fn session() -> Session {
        test::TestRequest::default().to_http_request().get_session()
    }

    async fn json(response: HttpResponse) -> serde_json::Value {
        let bytes = to_bytes(response.into_body()).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[actix_web::test]
    async fn mint_returns_a_public_url_and_expiry_for_a_stored_episode() {
        let response = create_share(
            state(fixture().await),
            session(),
            Path::from("abc123".to_string()),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = json(response).await;
        let url = body["data"]["url"].as_str().unwrap();
        assert!(url.starts_with("http://localhost:6996/app/share/"));
        assert!(body["data"]["expires_at"].is_string());
        let token = url.rsplit('/').next().unwrap();
        let verified = verify_token(
            &test_config().secret_key,
            token,
            Utc::now().timestamp(),
        )
        .expect("minted token must verify");
        assert_eq!(verified.yt_id, "abc123");
    }

    #[actix_web::test]
    async fn mint_unknown_episode_is_not_found() {
        let response = create_share(
            state(fixture().await),
            session(),
            Path::from("missing".to_string()),
        )
        .await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn share_route_requires_a_session() {
        let pool = fixture().await;
        let app = test::init_service(
            App::new()
                .app_data(state(pool))
                .wrap(
                    SessionMiddleware::builder(CookieSessionStore::default(), Key::generate())
                        .cookie_secure(false)
                        .build(),
                )
                .service(
                    web::scope("")
                        .wrap(RequireSession)
                        .service(
                            web::resource("/episodes/{yt_id}/share/")
                                .route(web::post().to(create_share)),
                        ),
                ),
        )
        .await;

        let response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/episodes/abc123/share/")
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn metadata_exposes_only_public_fields_for_a_valid_token() {
        let state = state(fixture().await);
        let (token, _) = mint_token(&state.config.secret_key, "abc123", 30);
        let response = get_shared_episode(state, Path::from(token.clone())).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = json(response).await;
        assert_eq!(body["title"], "Episode");
        assert_eq!(body["channel_title"], "Channel");
        assert!(body.get("user").is_none());
        assert!(body.get("progress").is_none());
        assert!(body.get("favorite").is_none());
        assert!(body
            .get("audio_url")
            .and_then(|value| value.as_str())
            .unwrap()
            .ends_with("/audio.mp3"));
    }

    #[actix_web::test]
    async fn metadata_rejects_malformed_forged_and_expired_tokens() {
        let state = state(fixture().await);
        let secret = state.config.secret_key.clone();
        let now = Utc::now().timestamp();

        let malformed = get_shared_episode(state.clone(), Path::from("garbage".to_string())).await;
        assert_eq!(malformed.status(), StatusCode::NOT_FOUND);

        let forged = get_shared_episode(
            state.clone(),
            Path::from(format!("{}-abc123-{}", now + 3600, "0".repeat(64))),
        )
        .await;
        assert_eq!(forged.status(), StatusCode::NOT_FOUND);

        let expired = get_shared_episode(
            state,
            Path::from(mint_token_with_exp(&secret, "abc123", now - 10)),
        )
        .await;
        assert_eq!(expired.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn audio_serves_with_range_head_and_private_cache() {
        let slug = format!("share_audio_{}", rand::random::<u64>());
        let directory = write_media(&slug, "abc123.mp3", b"0123456789");
        let pool = fixture().await;
        set_channel_slug(&pool, &slug).await;
        let state = state(pool);
        let (token, _) = mint_token(&state.config.secret_key, "abc123", 30);

        let full = get_shared_audio(media_request(), state.clone(), Path::from(token.clone())).await;
        assert_eq!(full.status(), StatusCode::OK);
        assert_eq!(
            full.headers().get(header::CACHE_CONTROL).unwrap(),
            "private"
        );
        assert_eq!(
            to_bytes(full.into_body()).await.unwrap(),
            b"0123456789".as_slice()
        );

        let ranged =
            get_shared_audio(media_request_with_range("bytes=2-5"), state.clone(), Path::from(token.clone()))
                .await;
        assert_eq!(ranged.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(
            ranged.headers().get(header::CONTENT_RANGE).unwrap(),
            "bytes 2-5/10"
        );
        assert_eq!(
            to_bytes(ranged.into_body()).await.unwrap(),
            b"2345".as_slice()
        );

        let head = get_shared_audio(media_head_request(), state, Path::from(token)).await;
        assert_eq!(head.status(), StatusCode::OK);
        assert_eq!(head.headers().get(header::CONTENT_LENGTH).unwrap(), "10");

        std::fs::remove_dir_all(directory).unwrap();
    }

    #[actix_web::test]
    async fn audio_serves_the_processed_derivative_when_sponsorblock_is_enabled() {
        let slug = format!("share_processed_{}", rand::random::<u64>());
        let directory = write_media(&slug, "abc123.mp3", b"original");
        std::fs::write(
            directory.join("abc123.sponsorblock.abcdef.mp3"),
            b"processed",
        )
        .unwrap();
        let pool = fixture().await;
        set_channel_slug(&pool, &slug).await;
        let episode_id: i64 = sqlx::query_scalar("SELECT id FROM episodes WHERE yt_id = 'abc123'")
            .fetch_one(&pool)
            .await
            .unwrap();
        SponsorBlockCache::upsert_success(
            &pool,
            episode_id,
            &[],
            "snapshot",
            "processing",
            Some("abc123.sponsorblock.abcdef.mp3"),
            Some(50.0),
        )
        .await
        .unwrap();
        let mut config = test_config();
        config.sponsorblock_enabled = true;
        let state = state_with(pool, config);
        let (token, _) = mint_token(&state.config.secret_key, "abc123", 30);

        let response = get_shared_audio(media_request(), state, Path::from(token)).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            to_bytes(response.into_body()).await.unwrap(),
            b"processed".as_slice()
        );

        std::fs::remove_dir_all(directory).unwrap();
    }

    #[actix_web::test]
    async fn audio_rejects_expired_tokens_and_missing_media() {
        let state = state(fixture().await);
        let expired = mint_token_with_exp(
            &state.config.secret_key,
            "abc123",
            Utc::now().timestamp() - 10,
        );
        let response =
            get_shared_audio(media_request(), state.clone(), Path::from(expired)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Valid, unexpired token, but no media file on disk for the episode.
        let (valid, _) = mint_token(&state.config.secret_key, "abc123", 30);
        let missing = get_shared_audio(media_request(), state, Path::from(valid)).await;
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn public_share_scope_serves_a_valid_token_without_credentials() {
        let slug = format!("share_scope_{}", rand::random::<u64>());
        let directory = write_media(&slug, "abc123.mp3", b"audio-bytes");
        let pool = fixture().await;
        set_channel_slug(&pool, &slug).await;
        let state = state(pool);
        // The point of the share surface: it must work while the rest of the
        // deployment is credential-protected.
        assert!(state.config.with_authentication);
        let (token, _) = mint_token(&state.config.secret_key, "abc123", 30);

        let app = test::init_service(
            App::new().app_data(state).service(
                web::scope("/s")
                    .service(
                        web::resource("/{token}/episode.json")
                            .route(web::get().to(get_shared_episode)),
                    )
                    .service(
                        web::resource("/{token}/audio.mp3")
                            .route(web::get().to(get_shared_audio))
                            .route(web::head().to(get_shared_audio)),
                    ),
            ),
        )
        .await;

        let metadata = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&format!("/s/{token}/episode.json"))
                .to_request(),
        )
        .await;
        assert_eq!(metadata.status(), StatusCode::OK);
        assert!(metadata.headers().get(header::WWW_AUTHENTICATE).is_none());

        let audio = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&format!("/s/{token}/audio.mp3"))
                .to_request(),
        )
        .await;
        assert_eq!(audio.status(), StatusCode::OK);
        assert!(audio.headers().get(header::WWW_AUTHENTICATE).is_none());

        let invalid = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/s/not-a-token/audio.mp3")
                .to_request(),
        )
        .await;
        assert_eq!(invalid.status(), StatusCode::NOT_FOUND);
        assert!(invalid.headers().get(header::WWW_AUTHENTICATE).is_none());

        std::fs::remove_dir_all(directory).unwrap();
    }
}
