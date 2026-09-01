use actix_session::Session;
use actix_web::{
    delete, get, post, put,
    web::{Data, Json, Path},
    Responder,
};
use serde::Deserialize;
use tracing::{debug, error, info};

use super::{
    super::{
        models::{audios_dir, images_dir, CResponse, Channel, NewChannel, UpdateChannel},
        utils::worker::update_channel as refresh_channel,
    },
    AppState,
};

#[get("/channels/")]
async fn read_all(data: Data<AppState>, session: Session) -> impl Responder {
    info!("read_all");
    match Channel::read_all(&data.pool).await {
        Ok(channels) => Ok(CResponse::ok(session, channels)),
        Err(mut e) => {
            error!("Error: {e}");
            e.set_session(session);
            Err(e)
        }
    }
}

#[post("/channels/")]
async fn create(
    data: Data<AppState>,
    session: Session,
    channel: Json<NewChannel>,
) -> impl Responder {
    info!("create");
    match Channel::new(&data.pool, channel.into_inner()).await {
        Ok(channel) => {
            let pool = data.pool.clone();
            let config = data.config.clone();
            let id = channel.id;
            actix_web::rt::spawn(async move {
                if let Err(e) = refresh_channel(&pool, id, &config).await {
                    error!("Cant refresh new channel {}: {}", id, e);
                }
            });
            Ok(CResponse::ok(session, channel))
        }
        Err(mut e) => {
            error!("Error: {e}");
            e.set_session(session);
            Err(e)
        }
    }
}

#[post("/channels/{channel}/update/")]
async fn update_episodes(
    data: Data<AppState>,
    session: Session,
    path: Path<String>,
) -> impl Responder {
    info!("update_episodes");
    let key = path.into_inner();
    match Channel::read_by_id_or_slug(&data.pool, &key).await {
        Ok(channel) => {
            let pool = data.pool.clone();
            let config = data.config.clone();
            let id = channel.id;
            actix_web::rt::spawn(async move {
                if let Err(e) = refresh_channel(&pool, id, &config).await {
                    error!("Cant refresh channel {}: {}", id, e);
                }
            });
            Ok(CResponse::ok(session, channel))
        }
        Err(mut e) => {
            error!("Error: {e}");
            e.set_session(session);
            Err(e)
        }
    }
}

#[post("/channels/{channel}/image/")]
async fn refresh_image(
    data: Data<AppState>,
    session: Session,
    path: Path<String>,
) -> impl Responder {
    info!("refresh_image");
    let key = path.into_inner();
    match Channel::read_by_id_or_slug(&data.pool, &key).await {
        Ok(channel) => {
            let url = channel.url.clone();
            match Channel::update_image(&data.pool, channel.id, &url).await {
                Ok(channel) => Ok(CResponse::ok(session, channel)),
                Err(mut e) => {
                    error!("Error: {e}");
                    e.set_session(session);
                    Err(e)
                }
            }
        }
        Err(mut e) => {
            error!("Error: {e}");
            e.set_session(session);
            Err(e)
        }
    }
}

#[put("/channels/{channel}/")]
async fn update(
    data: Data<AppState>,
    session: Session,
    path: Path<String>,
    channel: Json<UpdateChannel>,
) -> impl Responder {
    info!("update");
    let key = path.into_inner();
    let mut channel = channel.into_inner();
    match Channel::read_by_id_or_slug(&data.pool, &key).await {
        Ok(existing) => {
            channel.id = existing.id;
            match Channel::update(&data.pool, &channel).await {
                Ok(channel) => Ok(CResponse::ok(session, channel)),
                Err(mut e) => {
                    error!("Error: {e}");
                    e.set_session(session);
                    Err(e)
                }
            }
        }
        Err(mut e) => {
            error!("Error: {e}");
            e.set_session(session);
            Err(e)
        }
    }
}

#[get("/channels/{channel}/")]
async fn read(data: Data<AppState>, session: Session, path: Path<String>) -> impl Responder {
    info!("read");
    let key = path.into_inner();
    match Channel::read_by_id_or_slug(&data.pool, &key).await {
        Ok(channel) => Ok(CResponse::ok(session, channel)),
        Err(mut e) => {
            error!("Error: {e}");
            e.set_session(session);
            Err(e)
        }
    }
}
#[delete("/channels/{channel}/")]
async fn delete(data: Data<AppState>, session: Session, path: Path<String>) -> impl Responder {
    info!("delete");
    let key = path.into_inner();
    match Channel::read_by_id_or_slug(&data.pool, &key).await {
        Ok(channel) => {
            let folder = audios_dir();
            // Ownership guard: only remove the audio directory when no other
            // channel row references the same slug path. Post-migration the
            // slug is unique, but the guard covers stale/edge states so a
            // shared directory can never be wiped while another channel's
            // rows survive.
            match Channel::count_by_slug(&data.pool, &channel.slug).await {
                Ok(count) if count > 1 => {
                    error!(
                            "Channel slug `{}` still referenced by {} channels; skipping directory removal to avoid wiping another channel's audio",
                            &channel.slug, count
                        );
                }
                _ => {
                    info!("Remove directory {}/{}", folder, &channel.slug);
                    match tokio::fs::remove_dir_all(format!("{}/{}", folder, channel.slug)).await {
                        Ok(_) => debug!("Removed directorio {}/{}", folder, &channel.slug),
                        Err(e) => {
                            error!("Can't remove directory {}/{}: {}", folder, &channel.slug, e)
                        }
                    };
                    // Remove the cached cover image alongside the audio
                    // directory (channel-image-cache). Missing file is not
                    // an error: the channel may never have had a cached
                    // image.
                    match tokio::fs::remove_file(format!("{}/{}.jpg", images_dir(), channel.slug))
                        .await
                    {
                        Ok(_) => debug!("Removed cached image for {}", &channel.slug),
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                        Err(e) => error!("Can't remove cached image for {}: {}", &channel.slug, e),
                    };
                }
            }
            match Channel::delete(&data.pool, channel.id).await {
                Ok(channel) => Ok(CResponse::ok(session, channel)),
                Err(mut e) => {
                    error!("Error: {e}");
                    e.set_session(session);
                    Err(e)
                }
            }
        }
        Err(mut e) => {
            error!("Error: {e}");
            e.set_session(session);
            Err(e)
        }
    }
}

#[derive(Deserialize)]
struct PlaybackSpeedBody {
    playback_speed: f64,
}

#[put("/channels/{channel}/playback_speed/")]
async fn update_playback_speed(
    data: Data<AppState>,
    session: Session,
    path: Path<String>,
    body: Json<PlaybackSpeedBody>,
) -> actix_web::HttpResponse {
    info!("update_playback_speed");
    match Channel::set_playback_speed(
        &data.pool,
        &path.into_inner(),
        body.into_inner().playback_speed,
    )
    .await
    {
        // Fire-and-forget like the progress write: the 204 alone confirms the
        // write, and the error status (400 invalid value / 404 unknown
        // channel) is surfaced through the response line (per-channel-playback-speed).
        Ok(_) => actix_web::HttpResponse::NoContent().finish(),
        Err(e) => {
            error!("Error updating playback speed: {e}");
            CResponse::ko(e.status_code(), session)
        }
    }
}

#[cfg(test)]
mod handler_tests {
    use actix_session::{storage::CookieSessionStore, SessionMiddleware};
    use actix_web::cookie::Key;
    use actix_web::test;
    use actix_web::{web, App};
    use chrono::Utc;
    use sqlx::query;
    use sqlx::sqlite::SqliteRow;
    use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions, Row};
    use std::path::Path;

    use super::*;
    use crate::models::config::test_config;

    async fn memory_pool() -> sqlx::Pool<sqlx::Sqlite> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("memory pool");
        let migrations = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
        Migrator::new(migrations)
            .await
            .expect("load migrations")
            .run(&pool)
            .await
            .expect("run migrations");
        pool
    }

    async fn insert_channel(pool: &sqlx::Pool<sqlx::Sqlite>, slug: &str) -> i64 {
        let now = Utc::now();
        query(
            "INSERT INTO channels (url, title, slug, active, description, image, \
             first, max, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id",
        )
        .bind(format!("https://example.com/{slug}"))
        .bind(slug)
        .bind(slug)
        .bind(true)
        .bind("")
        .bind("")
        .bind(now)
        .bind(5i64)
        .bind(now)
        .bind(now)
        .map(|row: SqliteRow| row.get::<i64, _>("id"))
        .fetch_one(pool)
        .await
        .expect("insert channel")
    }

    #[actix_web::test]
    async fn updates_playback_speed_with_204_and_persists() {
        let pool = memory_pool().await;
        let channel_id = insert_channel(&pool, "c1").await;
        let app = test::init_service(
            App::new()
                .app_data(Data::new(AppState {
                    config: test_config(),
                    pool: pool.clone(),
                }))
                .wrap(
                    SessionMiddleware::builder(CookieSessionStore::default(), Key::generate())
                        .cookie_secure(false)
                        .build(),
                )
                .service(web::scope("/api/1.0").service(update_playback_speed)),
        )
        .await;

        let request = test::TestRequest::put()
            .uri("/api/1.0/channels/c1/playback_speed/")
            .set_json(serde_json::json!({ "playback_speed": 1.35 }))
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::NO_CONTENT);

        let stored: f64 = query("SELECT playback_speed FROM channels WHERE id = $1")
            .bind(channel_id)
            .map(|row: SqliteRow| row.get(0))
            .fetch_one(&pool)
            .await
            .expect("read stored speed");
        assert_eq!(stored, 1.35);
    }

    #[actix_web::test]
    async fn channel_list_returns_every_channel_for_client_side_pagination() {
        let pool = memory_pool().await;
        for slug in ["c1", "c2", "c3", "dot_csv_lab"] {
            insert_channel(&pool, slug).await;
        }
        let config = test_config();
        assert_eq!(
            config.per_page, 3,
            "fixture must reproduce the pagination boundary"
        );
        let app = test::init_service(
            App::new()
                .app_data(Data::new(AppState {
                    config,
                    pool: pool.clone(),
                }))
                .wrap(
                    SessionMiddleware::builder(CookieSessionStore::default(), Key::generate())
                        .cookie_secure(false)
                        .build(),
                )
                .service(web::scope("/api/1.0").service(read_all)),
        )
        .await;

        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/1.0/channels/")
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(response).await;
        let channels = body["data"].as_array().expect("channel list");
        assert_eq!(channels.len(), 4);
        assert!(channels
            .iter()
            .any(|channel| channel["slug"] == "dot_csv_lab"));
    }

    #[actix_web::test]
    async fn rejects_out_of_range_speeds_with_400() {
        let pool = memory_pool().await;
        insert_channel(&pool, "c1").await;
        let app = test::init_service(
            App::new()
                .app_data(Data::new(AppState {
                    config: test_config(),
                    pool: pool.clone(),
                }))
                .wrap(
                    SessionMiddleware::builder(CookieSessionStore::default(), Key::generate())
                        .cookie_secure(false)
                        .build(),
                )
                .service(web::scope("/api/1.0").service(update_playback_speed)),
        )
        .await;

        for speed in [0.2, 4.0] {
            let request = test::TestRequest::put()
                .uri("/api/1.0/channels/c1/playback_speed/")
                .set_json(serde_json::json!({ "playback_speed": speed }))
                .to_request();
            let response = test::call_service(&app, request).await;
            assert_eq!(
                response.status(),
                actix_web::http::StatusCode::BAD_REQUEST,
                "speed {speed}"
            );
        }
    }

    #[actix_web::test]
    async fn unknown_channel_answers_404() {
        let pool = memory_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(Data::new(AppState {
                    config: test_config(),
                    pool: pool.clone(),
                }))
                .wrap(
                    SessionMiddleware::builder(CookieSessionStore::default(), Key::generate())
                        .cookie_secure(false)
                        .build(),
                )
                .service(web::scope("/api/1.0").service(update_playback_speed)),
        )
        .await;

        let request = test::TestRequest::put()
            .uri("/api/1.0/channels/missing-channel/playback_speed/")
            .set_json(serde_json::json!({ "playback_speed": 1.5 }))
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::NOT_FOUND);
    }
}
