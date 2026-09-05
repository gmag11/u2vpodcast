use actix_session::Session;
use actix_web::{
    http::StatusCode,
    web::{Data, Json},
    HttpRequest, Responder,
};
use tracing::{error, info};

use crate::models::CResponse;
use crate::utils::rate_limit::LoginRateLimiter;

use super::{
    super::{
        models::User,
        utils::{USER_ACTIVE_KEY, USER_ID_KEY, USER_NAME_KEY, USER_ROLE_KEY},
    },
    AppState, Credentials,
};

// Shared failed-login limiter (brute-force protection). Keyed by client IP +
// username so one compromised address cannot lock out unrelated attempts.
static LOGIN_LIMITER: std::sync::OnceLock<LoginRateLimiter> = std::sync::OnceLock::new();

fn login_limiter() -> &'static LoginRateLimiter {
    LOGIN_LIMITER.get_or_init(LoginRateLimiter::new)
}

pub async fn get_session(session: Session) -> impl Responder {
    info!("get_session");
    info!("Session status: {:?}", session.status());
    CResponse::ok(session, "")
}

pub async fn post_login(
    req: HttpRequest,
    data: Data<AppState>,
    Json(credentials): Json<Credentials>,
    session: Session,
) -> impl Responder {
    info!("post_login");
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();
    let key = format!("{ip}|{}", credentials.username);
    // Brute-force guard: too many recent failures for this (IP, username)
    // block the attempt regardless of the submitted password.
    if login_limiter().is_blocked(&key) {
        error!(
            "Login attempt blocked for user `{}` (too many failures from {ip})",
            credentials.username
        );
        return CResponse::ko(StatusCode::TOO_MANY_REQUESTS, session);
    }
    match User::get_by_name(&data.pool, &credentials.username).await {
        Ok(user) => {
            if user.active && user.check_password(&credentials.password).await {
                info!("Ok");
                login_limiter().record_success(&key);
                session.renew();
                let insert_err = session
                    .insert(USER_ID_KEY, user.id)
                    .err()
                    .or_else(|| session.insert(USER_NAME_KEY, &user.name).err())
                    .or_else(|| session.insert(USER_ROLE_KEY, &user.role).err())
                    .or_else(|| session.insert(USER_ACTIVE_KEY, user.active).err());
                if let Some(e) = insert_err {
                    error!("Cannot populate session: {e}");
                    CResponse::ko(StatusCode::INTERNAL_SERVER_ERROR, session)
                } else {
                    CResponse::ok(session, "")
                }
            } else {
                error!("Unauthorized");
                login_limiter().record_failure(&key);
                CResponse::ko(StatusCode::UNAUTHORIZED, session)
            }
        }
        Err(_) => {
            // Same generic failure path as a wrong password, so usernames are
            // not enumerable through timing/response differences.
            login_limiter().record_failure(&key);
            CResponse::ko(StatusCode::UNAUTHORIZED, session)
        }
    }
}