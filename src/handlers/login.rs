use actix_web::{
    Responder,
    http::StatusCode,
    web::{
        Json,
        Data,
    },
};
use actix_session::Session;
use tracing::{info, error};

use crate::models::CResponse;

use super::{
    Credentials,
    AppState,
    super::{
        models::User,
        utils::{
            USER_ID_KEY,
            USER_NAME_KEY,
            USER_ROLE_KEY,
            USER_ACTIVE_KEY,
        }
    }
};

pub async fn get_session(
    session: Session,
) -> impl Responder{
    info!("get_session");
    info!("Session status: {:?}", session.status());
    CResponse::ok(session, "")
}

pub async fn post_login(
    data: Data<AppState>,
    Json(credentials): Json<Credentials>,
    session: Session,
) -> impl Responder{
    info!("post_login");
    match User::get_by_name(&data.pool, &credentials.username).await{
        Ok(user) => {
            if user.active && user.check_password(&credentials.password).await {
                info!("Ok");
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
            }else{
                error!("Unauthorized");
                CResponse::ko(StatusCode::UNAUTHORIZED, session)
            }
        },
        Err(_) => CResponse::ko(StatusCode::UNAUTHORIZED, session)
    }
}
