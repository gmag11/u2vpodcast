use actix_session::Session;
use actix_web::{get, web::Data, Responder};
use serde::Serialize;
use tracing::info;

use super::{super::models::CResponse, AppState};

#[derive(Serialize)]
struct ConfigResponse {
    per_page: i64,
}

#[get("/config/")]
async fn get_config(data: Data<AppState>, session: Session) -> impl Responder {
    info!("get_config");
    let per_page = data.config.per_page;
    CResponse::ok(session, ConfigResponse { per_page })
}
