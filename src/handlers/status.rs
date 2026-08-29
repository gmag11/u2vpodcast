use crate::models::CResponse;
use actix_session::Session;
use actix_web::Responder;

pub async fn get_status(session: Session) -> impl Responder {
    CResponse::ok(session, "Up and running")
}
