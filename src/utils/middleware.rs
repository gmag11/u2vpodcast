use std::{
    future::{ready, Future, Ready},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use actix_session::Session;
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
use sqlx::SqlitePool;

use crate::models::{
    AppState,
    CustomResponse,
    User,
    from_session,
};
use crate::utils::{
    USER_ID_KEY,
    USER_NAME_KEY,
    USER_ROLE_KEY,
    USER_ACTIVE_KEY,
};

type BoxFuture =
    Pin<Box<dyn Future<Output = Result<ServiceResponse<BoxBody>, Error>>>>;

// Revalidates a session claim set against the current `users` table on every
// request. A cookie that resolves to a missing or deactivated user is rejected
// (401), closing the gap where stale claims kept working until the cookie TTL.
// On success the claims (name/role/active) are refreshed from the DB row so the
// session always reflects the current state.
async fn validate_session(session: &Session, pool: &SqlitePool) -> bool {
    let claims = match from_session(session.clone()) {
        Ok(claims) => claims,
        Err(_) => return false,
    };
    let user = match User::read(pool, claims.id).await {
        Ok(user) => user,
        Err(_) => return false, // user deleted → reject
    };
    if !user.active {
        return false; // user deactivated → reject
    }
    // Refresh claims so a role/name/active change never lingers in the cookie.
    let _ = session.insert(USER_ID_KEY, user.id);
    let _ = session.insert(USER_NAME_KEY, &user.name);
    let _ = session.insert(USER_ROLE_KEY, &user.role);
    let _ = session.insert(USER_ACTIVE_KEY, user.active);
    true
}

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
        let pool = req.app_data::<Data<AppState>>().map(|data| data.pool.clone());
        let service = Rc::clone(&self.service);
        Box::pin(async move {
            let valid = match pool {
                Some(pool) => validate_session(&session, &pool).await,
                None => false,
            };
            if valid {
                let res = service.call(req).await?;
                Ok(res.map_into_boxed_body())
            } else {
                let response = CustomResponse::<Value>::new(
                    StatusCode::UNAUTHORIZED,
                    "Unauthorized",
                    Some(session),
                    None,
                );
                let http_response = HttpResponse::build(StatusCode::UNAUTHORIZED).json(response);
                Ok(req.into_response(http_response))
            }
        })
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
        let session = req.get_session();
        let service = Rc::clone(&self.service);
        let pool = data.pool.clone();
        Box::pin(async move {
            // Session branch: revalidated against the DB on every request.
            if validate_session(&session, &pool).await {
                let res = service.call(req).await?;
                return Ok(res.map_into_boxed_body());
            }
            // Fallback: HTTP Basic Auth, resolved against the DB.
            let mut payload = Payload::None;
            let extraction = BasicAuth::from_request(req.request(), &mut payload);
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