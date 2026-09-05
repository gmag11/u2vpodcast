use crate::models::CResponse;
use actix_session::Session;
use actix_web::{http::StatusCode, Responder};
use tracing::{error, info};

use super::super::utils::USER_ID_KEY;

pub async fn post_logout(session: Session) -> impl Responder {
    info!("post_logout");
    match session_user_id(&session).await {
        Ok(_) => {
            info!("Logout");
            session.clear();
            session.purge();
            CResponse::purge()
        }
        Err(e) => {
            error!("Error: {}", e);
            CResponse::ko(StatusCode::BAD_REQUEST, session)
        }
    }
}

async fn session_user_id(session: &actix_session::Session) -> Result<i64, String> {
    match session.get(USER_ID_KEY) {
        Ok(user_id) => match user_id {
            None => Err("You are not authenticated".to_string()),
            Some(id) => Ok(id),
        },
        Err(e) => Err(format!("{e}")),
    }
}

#[cfg(test)]
mod logout_route_tests {
    use super::*;
    use actix_session::{storage::CookieSessionStore, SessionMiddleware};
    use actix_web::cookie::Key;
    use actix_web::{http::header, test, web, App};
    use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions};
    use std::path::Path;

    use crate::models::config::test_config;
    use crate::utils::csrf::CsrfProtection;

    async fn memory_pool() -> sqlx::Pool<sqlx::Sqlite> {
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
        pool
    }

    async fn call_logout(
        method: actix_web::http::Method,
        origin: Option<&str>,
    ) -> actix_web::http::StatusCode {
        let pool = memory_pool().await;
        let data = web::Data::new(crate::models::AppState {
            config: test_config(),
            pool,
        });
        let app = test::init_service(
            App::new()
                .app_data(data)
                .wrap(
                    SessionMiddleware::builder(CookieSessionStore::default(), Key::generate())
                        .cookie_secure(false)
                        .build(),
                )
                .wrap(CsrfProtection)
                .service(
                    web::scope("/api/1.0")
                        .service(web::resource("/logout/").route(web::post().to(post_logout))),
                ),
        )
        .await;
        let mut req = test::TestRequest::default()
            .method(method.clone())
            .uri("/api/1.0/logout/");
        if let Some(origin) = origin {
            req = req.insert_header((header::ORIGIN, origin));
        }
        let response = test::call_service(&app, req.to_request()).await;
        response.status()
    }

    #[actix_web::test]
    async fn logout_is_post_only_not_get() {
        // A GET (the old cross-site-loggable verb) must be rejected outright.
        let status = call_logout(actix_web::http::Method::GET, None).await;
        assert_eq!(status, actix_web::http::StatusCode::METHOD_NOT_ALLOWED);
    }

    #[actix_web::test]
    async fn logout_post_from_cross_site_origin_is_rejected() {
        let status = call_logout(
            actix_web::http::Method::POST,
            Some("https://evil.example.com"),
        )
        .await;
        assert_eq!(status, actix_web::http::StatusCode::FORBIDDEN);
    }

    #[actix_web::test]
    async fn logout_post_without_origin_reaches_the_handler() {
        // No session -> the handler answers 400 "not authenticated", proving
        // the POST reached the handler (not blocked by CSRF/CORS).
        let status = call_logout(actix_web::http::Method::POST, None).await;
        assert_eq!(status, actix_web::http::StatusCode::BAD_REQUEST);
    }
}
