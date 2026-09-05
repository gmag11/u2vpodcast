use std::{
    future::{ready, Future, Ready},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::{header, Method},
    web::Data,
    Error, HttpResponse,
};

use crate::models::AppState;

type BoxFuture = Pin<Box<dyn Future<Output = Result<ServiceResponse<BoxBody>, Error>>>>;

// Safe (read-only) methods never mutate state, so they are exempt from the
// origin check: GET/HEAD are how feeds, media and the API reads are consumed,
// and OPTIONS is the CORS preflight already handled by the CORS layer.
fn is_safe_method(method: &Method) -> bool {
    matches!(
        *method,
        Method::GET | Method::HEAD | Method::OPTIONS | Method::TRACE
    )
}

// Cross-site write protection for cookie-authenticated state changes. The
// session cookie is SameSite=None, so browsers happily send it with requests
// from any site; CORS only blocks reading the response, not sending the
// request. Checking the Origin header against the same explicit allowlist the
// CORS policy uses closes that CSRF hole for browser-originated requests while
// allowing non-browser clients (curl, podcast apps, ...) that omit Origin.
fn origin_is_allowed(origin: &str, allowed: &[String]) -> bool {
    let trimmed = origin.trim().trim_end_matches('/').to_lowercase();
    allowed
        .iter()
        .any(|allowed| allowed.trim_end_matches('/').to_lowercase() == trimmed)
}

pub struct CsrfProtection;

impl<S, B> Transform<S, ServiceRequest> for CsrfProtection
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = CsrfProtectionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CsrfProtectionMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct CsrfProtectionMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for CsrfProtectionMiddleware<S>
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
        let service = Rc::clone(&self.service);
        if is_safe_method(req.method()) {
            return Box::pin(async move {
                let res = service.call(req).await?;
                Ok(res.map_into_boxed_body())
            });
        }
        let allowed = req
            .app_data::<Data<AppState>>()
            .map(|data| data.config.cors_origins())
            .unwrap_or_default();
        Box::pin(async move {
            // No Origin header => non-browser client; allow. An Origin header
            // that is not in the allowlist => cross-site request; reject.
            let cross_site = req
                .headers()
                .get(header::ORIGIN)
                .and_then(|value| value.to_str().ok())
                .map(|origin| !origin_is_allowed(origin, &allowed))
                .unwrap_or(false);
            if cross_site {
                return Ok(req.into_response(
                    HttpResponse::Forbidden()
                        .body("Cross-site request rejected (Origin not allowed)"),
                ));
            }
            let res = service.call(req).await?;
            Ok(res.map_into_boxed_body())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_methods_are_exempt() {
        for method in [
            Method::GET,
            Method::HEAD,
            Method::OPTIONS,
            Method::TRACE,
        ] {
            assert!(is_safe_method(&method), "{method} must be safe");
        }
        for method in [Method::POST, Method::PUT, Method::DELETE, Method::PATCH] {
            assert!(!is_safe_method(&method), "{method} must be unsafe");
        }
    }

    #[test]
    fn origin_matching_is_case_and_trailing_slash_insensitive() {
        let allowed = vec!["https://podcasts.example.com".to_string()];
        assert!(origin_is_allowed("https://podcasts.example.com", &allowed));
        assert!(origin_is_allowed("https://podcasts.example.com/", &allowed));
        assert!(origin_is_allowed("HTTPS://PODCASTS.EXAMPLE.COM", &allowed));
        assert!(!origin_is_allowed("https://evil.example.com", &allowed));
        assert!(!origin_is_allowed("http://podcasts.example.com", &allowed));
        assert!(!origin_is_allowed("https://podcasts.example.com.evil.com", &allowed));
    }
}