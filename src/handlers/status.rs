use actix_session::Session;
use actix_web::Responder;
use crate::models::CResponse;

pub async fn get_status(
    session: Session
) -> impl Responder{
    CResponse::ok(session, "Up and running")
}

