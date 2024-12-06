use actix_service::forward_ready;
use actix_web::{
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::{
        header::{HeaderName, HeaderValue},
        Method,
    },
    web::Data,
    Error, HttpResponse,
};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use log::{debug, error};

use share_lib::{data_structure::MailManErr, log_debug, log_error};

use crate::models::user_token::{TokenModel, UserToken};

const IGNORE_ROUTES: [&str; 2] = ["/api/auth/signup", "/api/auth/login"];

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

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let mut authenticate_pass: bool = false;

        // Bypass some account routes
        let mut headers = req.headers().clone();
        headers.append(
            HeaderName::from_static("content-length"),
            HeaderValue::from_static("true"),
        );
        if Method::OPTIONS == *req.method() {
            authenticate_pass = true;
        } else {
            for ignore_route in IGNORE_ROUTES.iter() {
                if req.path().starts_with(ignore_route) {
                    authenticate_pass = true;
                    break;
                }
            }
        }

        if !authenticate_pass {
            if let Some(pool) = req.app_data::<Data<Pool<ConnectionManager<MysqlConnection>>>>() {
                log_debug!("Connecting to database...");
                if let Some(authen_header) = req.headers().get("Authorization") {
                    log_debug!("Parsing authorization header...");
                    if let Ok(authen_str) = authen_header.to_str() {
                        if authen_str.starts_with("bearer") || authen_str.starts_with("Bearer") {
                            log_debug!("Parsing token...");
                            let token = authen_str[6..authen_str.len()].trim();
                            if let Ok(token_data) = UserToken::decode_token(token.to_string()) {
                                log_debug!("Decoding token...");
                                if TokenModel::token_ckeck(&token_data, &mut pool.get().unwrap())
                                    .is_ok()
                                {
                                    log_debug!("Valid token");
                                    authenticate_pass = true;
                                } else {
                                    log_error!("Invalid token");
                                }
                            }
                        }
                    }
                }
            }
        }

        if !authenticate_pass {
            let (request, _pl) = req.into_parts();
            let response = HttpResponse::Unauthorized()
                .json(MailManErr::new(
                    401,
                    "Invalid token",
                    "please login again",
                    1,
                ))
                .map_into_right_body();

            return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
        }

        let res = self.service.call(req);

        Box::pin(async move { res.await.map(ServiceResponse::map_into_left_body) })
    }
}
