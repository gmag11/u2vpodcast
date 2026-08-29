use actix_session::Session;
use actix_web::{
    get, post, put,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use serde::Deserialize;
use tracing::{debug, error, info};

use super::{
    super::models::{audios_dir, CResponse, Channel, Episode, EpisodeProgress},
    AppState,
};
use crate::utils::sponsorblock::{reconcile_episode, SponsorBlockClient};
use std::path::Path as FsPath;

#[get("/channels/{channel}/episodes/")]
async fn read_with_pagination(
    data: Data<AppState>,
    session: Session,
    path: Path<String>,
) -> impl Responder {
    info!("read_api_channels");
    let key = path.into_inner();
    match Channel::read_by_id_or_slug(&data.pool, &key).await {
        Ok(channel) => match Episode::read_episodes_for_channel(&data.pool, channel.id).await {
            Ok(mut episodes) => {
                debug!("{:?}", episodes);
                for episode in episodes.iter_mut() {
                    episode.channel_slug = channel.slug.clone();
                    episode.playback_speed = channel.playback_speed;
                    episode.apply_sponsorblock_config(
                        data.config.sponsorblock_enabled,
                        &data.config.sponsorblock_rejected_categories,
                    );
                }
                Ok(CResponse::ok(session, episodes))
            }
            Err(e) => {
                error!("{e}");
                Err(e)
            }
        },
        Err(e) => {
            error!("{e}");
            Err(e)
        }
    }
}

#[get("/episodes/")]
async fn read_all(data: Data<AppState>, session: Session) -> impl Responder {
    info!("read_all");
    match Episode::read_all_with_channels(&data.pool).await {
        Ok(mut episodes) => {
            for episode in &mut episodes {
                episode.apply_sponsorblock_config(
                    data.config.sponsorblock_enabled,
                    &data.config.sponsorblock_rejected_categories,
                );
            }
            Ok(CResponse::ok(session, episodes))
        }
        Err(e) => {
            error!("{e}");
            Err(e)
        }
    }
}

#[derive(Deserialize)]
pub struct ProgressBody {
    pub position_seconds: i64,
    pub listened: bool,
}

#[derive(Deserialize)]
pub struct FavoriteBody {
    pub favorite: bool,
}

#[get("/episodes/{yt_id}/progress/")]
async fn read_progress(
    data: Data<AppState>,
    session: Session,
    yt_id: Path<String>,
) -> actix_web::HttpResponse {
    info!("read_progress");
    match Episode::read_progress_by_yt_id(&data.pool, &yt_id.into_inner()).await {
        Ok(progress) => {
            let progress: EpisodeProgress = progress;
            CResponse::ok(session, progress)
        }
        Err(e) => {
            // A missing episode is 404; any other failure (e.g. a real
            // database error) surfaces its own status instead of being masked.
            error!("Error reading progress: {e}");
            CResponse::ko(e.status_code(), session)
        }
    }
}

#[put("/episodes/{yt_id}/progress/")]
async fn update_progress(
    data: Data<AppState>,
    session: Session,
    yt_id: Path<String>,
    body: Json<ProgressBody>,
) -> actix_web::HttpResponse {
    info!("update_progress");
    let body = body.into_inner();
    // A position can never be negative: clamp rather than storing junk.
    let position_seconds = body.position_seconds.max(0);
    match Episode::update_progress_by_yt_id(
        &data.pool,
        &yt_id.into_inner(),
        position_seconds,
        body.listened,
    )
    .await
    {
        // The request is fire-and-forget: the 204 status alone confirms the
        // write; no response body is needed.
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            error!("Error updating progress: {e}");
            CResponse::ko(e.status_code(), session)
        }
    }
}

#[put("/episodes/{yt_id}/favorite/")]
async fn update_favorite(
    data: Data<AppState>,
    session: Session,
    yt_id: Path<String>,
    body: Json<FavoriteBody>,
) -> actix_web::HttpResponse {
    info!("update_favorite");
    match Episode::set_favorite_by_yt_id(
        &data.pool,
        &yt_id.into_inner(),
        body.into_inner().favorite,
    )
    .await
    {
        // Fire-and-forget like the progress write: the 204 alone confirms the
        // write; a missing episode surfaces its 404 through the error status.
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            error!("Error updating favorite: {e}");
            CResponse::ko(e.status_code(), session)
        }
    }
}

#[post("/episodes/{yt_id}/sponsorblock/refresh/")]
async fn refresh_sponsorblock(
    data: Data<AppState>,
    session: Session,
    yt_id: Path<String>,
) -> actix_web::HttpResponse {
    if !data.config.sponsorblock_enabled {
        return CResponse::ko_with_message(
            actix_web::http::StatusCode::CONFLICT,
            "SponsorBlock is disabled",
            session,
        );
    }
    match refresh_sponsorblock_episode(
        &data.pool,
        &SponsorBlockClient::default(),
        FsPath::new(audios_dir()),
        &yt_id.into_inner(),
        data.config.sponsorblock_enabled,
        &data.config.sponsorblock_rejected_categories,
    )
    .await
    {
        Ok(updated) => CResponse::ok(session, updated),
        Err(error) => {
            error!("Error refreshing SponsorBlock data: {error}");
            CResponse::ko(error.status_code(), session)
        }
    }
}

async fn refresh_sponsorblock_episode(
    pool: &sqlx::SqlitePool,
    client: &SponsorBlockClient,
    audio_root: &FsPath,
    yt_id: &str,
    sponsorblock_enabled: bool,
    rejected_categories: &[String],
) -> Result<Episode, super::super::models::Error> {
    if !sponsorblock_enabled {
        return Err(super::super::models::Error::new_with_status_code(
            "SponsorBlock is disabled",
            actix_web::http::StatusCode::CONFLICT,
        ));
    }
    let episode = Episode::read_by_yt_id_with_channel(pool, yt_id).await?;
    let channel_dir = audio_root.join(&episode.channel_slug);
    reconcile_episode(pool, client, &episode, &channel_dir, rejected_categories).await?;
    let mut episode = Episode::read_by_yt_id_with_channel(pool, yt_id).await?;
    episode.apply_sponsorblock_config(true, rejected_categories);
    Ok(episode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SponsorBlockCache;
    use actix_web::http::StatusCode;
    use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions};
    use std::{
        io::{Read, Write},
        net::TcpListener,
        path::PathBuf,
        thread,
        time::Duration,
    };

    async fn fixture() -> (sqlx::SqlitePool, PathBuf) {
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
        let old = chrono::Utc::now() - chrono::Duration::days(365);
        let channel_id: i64 = sqlx::query_scalar(
            "INSERT INTO channels (url, title, slug, active, description, image, first, max, created_at, updated_at) \
             VALUES ('https://example.com', 'Channel', 'channel', TRUE, '', '', $1, 5, $1, $1) RETURNING id",
        )
        .bind(old)
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO episodes (channel_id, title, yt_id, webpage_url, published_at, duration, favorite, created_at, updated_at) \
             VALUES ($1, 'Old favorite', 'old-favorite', 'https://example.com/video', $2, '00:03:00', TRUE, $2, $2)",
        )
        .bind(channel_id)
        .bind(old)
        .execute(&pool)
        .await
        .unwrap();
        let root = std::env::temp_dir().join(format!(
            "u2vpodcast-sponsorblock-handler-{}",
            rand::random::<u64>()
        ));
        std::fs::create_dir_all(root.join("channel")).unwrap();
        (pool, root)
    }

    fn empty_snapshot_server() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 2048];
            let read = stream.read(&mut request).unwrap();
            assert!(String::from_utf8_lossy(&request[..read]).contains("videoID=old-favorite"));
            stream
                .write_all(
                    b"HTTP/1.1 404 Not Found\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
                )
                .unwrap();
        });
        format!("http://{address}")
    }

    #[actix_web::test]
    async fn manual_refresh_accepts_an_old_favorite() {
        let (pool, root) = fixture().await;
        let client = SponsorBlockClient::new(&empty_snapshot_server(), Duration::from_secs(1));

        let refreshed = refresh_sponsorblock_episode(
            &pool,
            &client,
            &root,
            "old-favorite",
            true,
            &["sponsor".to_string()],
        )
        .await
        .unwrap();

        assert!(refreshed.favorite);
        assert!(refreshed.sponsorblock_enabled);
        assert!(refreshed.sponsorblock_segments.is_empty());
        assert!(SponsorBlockCache::read(&pool, refreshed.id)
            .await
            .unwrap()
            .is_some());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[actix_web::test]
    async fn manual_refresh_rejects_an_unknown_episode() {
        let (pool, root) = fixture().await;
        let client = SponsorBlockClient::new("http://127.0.0.1:1", Duration::from_millis(50));

        let error = refresh_sponsorblock_episode(
            &pool,
            &client,
            &root,
            "unknown",
            true,
            &["sponsor".to_string()],
        )
        .await
        .unwrap_err();

        assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[actix_web::test]
    async fn disabled_manual_refresh_stops_before_lookup_or_client_access() {
        let (pool, root) = fixture().await;
        let client = SponsorBlockClient::new("http://127.0.0.1:1", Duration::from_millis(10));
        let before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sponsorblock_cache")
            .fetch_one(&pool)
            .await
            .unwrap();

        let error = refresh_sponsorblock_episode(
            &pool,
            &client,
            &root,
            "old-favorite",
            false,
            &["sponsor".to_string()],
        )
        .await
        .unwrap_err();

        assert_eq!(error.status_code(), StatusCode::CONFLICT);
        assert!(error.to_string().contains("disabled"));
        let after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sponsorblock_cache")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(after, before);
        std::fs::remove_dir_all(root).unwrap();
    }
}
