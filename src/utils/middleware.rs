use std::{
    future::{ready, Future, Ready},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use actix_session::SessionExt;
use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform},
    http::{
        header::WWW_AUTHENTICATE,
        StatusCode,
    },
    web::Data,
    Error,
    FromRequest,
    HttpResponse,
};
use actix_web_httpauth::extractors::basic::BasicAuth;
use serde_json::Value;

use crate::models::{
    AppState,
    CustomResponse,
    User,
    from_session,
};

type BoxFuture =
    Pin<Box<dyn Future<Output = Result<ServiceResponse<BoxBody>, Error>>>>;

// ---------- require_session ----------

pub struct RequireSession;

impl<S, B> Transform<S, ServiceRequest> for RequireSession
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireSessionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireSessionMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct RequireSessionMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequireSessionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = BoxFuture;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let session = req.get_session();
        match from_session(session.clone()) {
            Ok(_) => {
                let service = Rc::clone(&self.service);
                Box::pin(async move {
                    let res = service.call(req).await?;
                    Ok(res.map_into_boxed_body())
                })
            }
            Err(_) => {
                let response = CustomResponse::<Value>::new(
                    StatusCode::UNAUTHORIZED,
                    "Unauthorized",
                    Some(session),
                    None,
                );
                let http_response = HttpResponse::build(StatusCode::UNAUTHORIZED).json(response);
                Box::pin(async move { Ok(req.into_response(http_response)) })
            }
        }
    }
}

// ---------- session_or_basic ----------

fn unauthorized(req: ServiceRequest) -> ServiceResponse<BoxBody> {
    req.into_response(
        HttpResponse::build(StatusCode::UNAUTHORIZED)
            .insert_header((WWW_AUTHENTICATE, "Basic realm=\"u2vpodcast\""))
            .finish(),
    )
}

pub struct SessionOrBasicAuth;

impl<S, B> Transform<S, ServiceRequest> for SessionOrBasicAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = SessionOrBasicAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SessionOrBasicAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct SessionOrBasicAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SessionOrBasicAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = BoxFuture;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let data = match req.app_data::<Data<AppState>>().cloned() {
            Some(data) => data,
            None => return Box::pin(async move { Ok(unauthorized(req)) }),
        };
        if !data.config.with_authentication {
            let service = Rc::clone(&self.service);
            return Box::pin(async move {
                let res = service.call(req).await?;
                Ok(res.map_into_boxed_body())
            });
        }
        if from_session(req.get_session()).is_ok() {
            let service = Rc::clone(&self.service);
            return Box::pin(async move {
                let res = service.call(req).await?;
                Ok(res.map_into_boxed_body())
            });
        }
        let mut payload = Payload::None;
        let extraction = BasicAuth::from_request(req.request(), &mut payload);
        let service = Rc::clone(&self.service);
        let pool = data.pool.clone();
        Box::pin(async move {
            let credentials = match extraction.await {
                Ok(credentials) => credentials,
                Err(_) => return Ok(unauthorized(req)),
            };
            let username = credentials.user_id().to_string();
            let password = credentials
                .password()
                .map(|password| password.to_string())
                .unwrap_or_default();
            match User::get_by_name(&pool, &username).await {
                Ok(user) => {
                    if user.active && user.check_password(&password).await {
                        let res = service.call(req).await?;
                        Ok(res.map_into_boxed_body())
                    } else {
                        Ok(unauthorized(req))
                    }
                }
                Err(_) => Ok(unauthorized(req)),
            }
        })
    }
}