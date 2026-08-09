use actix_session::Session;
use serde::Serialize;
use actix_web::{
    Responder,
    web::Data,
    get,
};
use tracing::info;

use super::{
    AppState,
    super::models::CResponse,
};

#[derive(Serialize)]
struct ConfigResponse {
    per_page: i64,
}

#[get("/config/")]
async fn get_config(
    data: Data<AppState>,
    session: Session,
) -> impl Responder{
    info!("get_config");
    let per_page = data.config.per_page;
    CResponse::ok(session, ConfigResponse { per_page })
}
