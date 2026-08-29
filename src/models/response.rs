use actix_session::Session;
use actix_web::{http::StatusCode, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::user::from_session;
use super::user::SessionUser;

#[derive(Debug, Deserialize, Serialize)]
pub struct CustomResponse<T> {
    pub status: bool,
    pub status_code: u16,
    pub message: String,
    pub user: Option<SessionUser>,
    pub data: Option<T>,
}

pub struct CResponse;

impl CResponse {
    pub fn ok(session: Session, data: impl Serialize) -> HttpResponse {
        let content = serde_json::to_value(data).unwrap();
        let response: CustomResponse<Value> =
            CustomResponse::new(StatusCode::OK, "Ok", Some(session), Some(content));
        HttpResponse::build(StatusCode::OK).json(response)
    }

    pub fn purge() -> HttpResponse {
        let response: CustomResponse<String> = CustomResponse {
            status: true,
            status_code: 200,
            message: "Ok".to_string(),
            user: None,
            data: None,
        };
        HttpResponse::build(StatusCode::OK).json(response)
    }

    pub fn ko(status_code: StatusCode, session: Session) -> HttpResponse {
        Self::ko_with_message(status_code, status_code.as_str(), session)
    }

    pub fn ko_with_message(
        status_code: StatusCode,
        message: &str,
        session: Session,
    ) -> HttpResponse {
        let user = from_session(session).ok();
        let response = CustomResponse::<Value> {
            status: status_code.is_success(),
            status_code: status_code.as_u16(),
            message: message.to_string(),
            user,
            data: None::<Value>,
        };
        // Report the failing status in the HTTP status line too, so clients do
        // not need to parse the body to detect errors (api-response-contract).
        HttpResponse::build(status_code).json(response)
    }
}

impl<T> CustomResponse<T> {
    pub fn new(
        status_code: StatusCode,
        message: &str,
        session: Option<Session>,
        data: Option<T>,
    ) -> CustomResponse<T> {
        let status_code = status_code.as_u16();
        let status = status_code < 300;
        let user = session.and_then(|session| from_session(session).ok());
        Self {
            status,
            status_code,
            message: message.to_string(),
            user,
            data,
        }
    }
}

#[cfg(test)]
mod response_envelope_tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn ko_envelope_marks_failure() {
        let resp =
            CustomResponse::<Value>::new(StatusCode::UNAUTHORIZED, "Unauthorized", None, None);
        assert!(!resp.status);
        assert_eq!(resp.status_code, 401);
        assert_eq!(resp.message, "Unauthorized");
    }

    #[test]
    fn ok_envelope_marks_success() {
        let resp = CustomResponse::<Value>::new(StatusCode::OK, "Ok", None, None);
        assert!(resp.status);
        assert_eq!(resp.status_code, 200);
    }
}

//impl<T> Into<HttpResponse> for CustomResponse<T>
//where T: DeserializeOwned + Serialize{
//    fn into(self) -> HttpResponse {
//        HttpResponse::build(StatusCode::from_u16(self.status_code).unwrap())
//            .json(self)
//    }
//}
