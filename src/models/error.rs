use actix_session::Session;
use actix_web::{http::StatusCode, Error as ActixError, HttpResponse, ResponseError};
use serde::{ser::SerializeStruct, Serialize};
use sqlx::{migrate::MigrateError, Error as SQLxError};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result},
    io::Error as IoError,
    num::ParseIntError,
    str::Utf8Error,
};

pub struct Error {
    details: String,
    session: Option<Session>,
    status_code: Option<StatusCode>,
}
use super::super::models::CustomResponse;

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("details", &self.details)
            .field("status_code", &self.status_code)
            .finish()
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Error", 2)?;
        state.serialize_field("details", &self.details)?;
        state.serialize_field("status_code", &self.status_code().as_u16())?;
        state.end()
    }
}

impl Error {
    pub fn set_session(&mut self, session: Session) {
        self.session = Some(session);
    }
    pub fn default(msg: &str) -> Self {
        Error {
            details: msg.to_string(),
            status_code: None,
            session: None,
        }
    }
    pub fn new(msg: &str, session: &Session) -> Self {
        Error {
            details: msg.to_string(),
            status_code: None,
            session: Some(session.clone()),
        }
    }

    pub fn new_with_status_code(msg: &str, status_code: StatusCode) -> Self {
        Error {
            details: msg.to_string(),
            status_code: Some(status_code),
            session: None,
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self.status_code {
            Some(status_code) => status_code,
            None => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.details)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<SQLxError> for Error {
    fn from(error: SQLxError) -> Self {
        Error::default(&error.to_string())
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::default(&error.to_string())
    }
}

impl From<ParseIntError> for Error {
    fn from(error: ParseIntError) -> Self {
        Error::default(&error.to_string())
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error::default(&error.to_string())
    }
}

impl From<MigrateError> for Error {
    fn from(error: MigrateError) -> Self {
        Error::default(&error.to_string())
    }
}

impl From<ActixError> for Error {
    fn from(error: ActixError) -> Self {
        Error::default(&error.to_string())
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        // Internal details (SQL errors, filesystem paths, library messages)
        // must never reach the client: they leak the database schema and
        // implementation internals. Log the detail server-side and return a
        // generic body for 5xx; 4xx messages are intentional validation
        // messages and are safe to expose.
        if self.status_code() == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!("Internal error: {}", self.details);
        }
        let message = if self.status_code() == StatusCode::INTERNAL_SERVER_ERROR {
            "Internal server error".to_string()
        } else {
            self.details.clone()
        };
        let response: CustomResponse<Option<String>> = CustomResponse::new(
            self.status_code(),
            &message,
            self.session.clone(),
            None,
        );
        HttpResponse::build(self.status_code()).json(response)
    }
}

#[cfg(test)]
mod error_response_tests {
    use super::*;
    use actix_web::{
        body::to_bytes,
        http::StatusCode,
    };

    #[actix_web::test]
    async fn internal_error_returns_generic_message_not_internal_details() {
        // A 500 from a SQL error must not leak the raw detail to the client.
        let error = Error::default("SQLite error: no such table: secret_users");
        let response = error.error_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = to_bytes(response.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["message"], "Internal server error");
        assert!(!body.windows(6).any(|w| w == b"secret"));
    }

    #[actix_web::test]
    async fn client_error_keeps_intentional_message() {
        let error = Error::new_with_status_code("max must be >= 1", StatusCode::BAD_REQUEST);
        let response = error.error_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["message"], "max must be >= 1");
    }
}
