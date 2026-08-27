use actix_service::forward_ready;
use actix_web::{
    Error, HttpResponse,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::{
        Method,
        header::{HeaderName, HeaderValue},
    },
};
use futures::future::{LocalBoxFuture, Ready, ok};

use share_lib::data_structure::MailManErr;

use crate::server::GLOBAL_CONFIG;

// used to crate a middleware
pub struct Authentication;

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthenticationMiddleware { service })
    }
}

// used to implement the middleware service
pub struct AuthenticationMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    // used to proxy the readiness state of the service.
    forward_ready!(service);

    // the authentication logic
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let mut authenticate_pass: bool = false;

        let mut internal_error: (bool, String) = (false, String::new());

        // Bypass some account routes
        let mut headers = req.headers().clone();
        headers.append(
            HeaderName::from_static("content-length"),
            HeaderValue::from_static("true"),
        );
        if Method::OPTIONS == *req.method() {
            authenticate_pass = true;
        } else {
            for ignore_route in GLOBAL_CONFIG.read().unwrap().authenticate_bypass.iter() {
                if req.path().starts_with(ignore_route) {
                    authenticate_pass = true;
                    break;
                }
            }
        }

        if !authenticate_pass {
            if let Ok(token_lock) = GLOBAL_CONFIG.read() {
                let auth_uuid = &token_lock.subsys_uuid;
                if let Some(authen_header) = req.headers().get("Authorization") {
                    if let Ok(authen_str) = authen_header.to_str() {
                        if authen_str.starts_with("uuid") || authen_str.starts_with("UUID") {
                            let token = authen_str[5..authen_str.len()].trim().to_string();
                            if *auth_uuid == token {
                                authenticate_pass = true;
                            } else {
                                internal_error = (true, "UUID dismatch".to_string())
                            }
                        }
                    }
                }
            }
        }

        if internal_error.0 {
            let (request, _pl) = req.into_parts();
            let response = HttpResponse::Unauthorized()
                .json(MailManErr::new(
                    500,
                    "Internal Server Error",
                    Some(internal_error.1),
                    1,
                ))
                .map_into_right_body();

            return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
        }

        if !authenticate_pass {
            let (request, _pl) = req.into_parts();
            let response = HttpResponse::Unauthorized()
                .json(MailManErr::new(
                    401,
                    "Invalid UUID",
                    Some("please login again".to_string()),
                    1,
                ))
                .map_into_right_body();

            return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
        }

        let res = self.service.call(req);

        Box::pin(async move { res.await.map(ServiceResponse::map_into_left_body) })
    }
}
