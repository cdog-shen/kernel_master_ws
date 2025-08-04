use actix_service::forward_ready;
use actix_web::{
    Error, HttpResponse,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::{
        Method,
        header::{HeaderName, HeaderValue},
    },
    web::Data,
};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use futures::future::{LocalBoxFuture, Ready, ok};
// use log::{debug, error};

use share_lib::data_structure::MailManErr;
// use share_lib::{log_debug, log_error};

use crate::{
    model::{
        access::AccessModel,
        group::GroupModel,
        service::ServiceModel,
        user::UserModel,
        user_token::{TokenModel, UserToken},
    },
    server::GLOBAL_CONFIG,
};

// those routes dose not need pass this middleware
// const AUTHENTICATE_BYPASS: [&str; 5] = [
//     "/api/auth/signup",
//     "/api/auth/login",
//     "/webhook",
//     "/api/hey",
//     "/api/reload",
// ];
// const PERMIT_BYPASS: [&str; 5] = [
//     "/api/auth/signup",
//     "/api/auth/login",
//     "/webhook",
//     "/api/hey",
//     "/api/reload",
// ];

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
        let mut permit_pass: bool = false;
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

        // Bypass some low permission route
        let mut headers = req.headers().clone();
        headers.append(
            HeaderName::from_static("content-length"),
            HeaderValue::from_static("true"),
        );
        if Method::OPTIONS == *req.method() {
            permit_pass = true;
        } else {
            for ignore_route in GLOBAL_CONFIG.read().unwrap().permit_bypass.iter() {
                if req.path().starts_with(ignore_route) {
                    permit_pass = true;
                    break;
                }
            }
        }

        if !authenticate_pass {
            if let Some(pool) = req.app_data::<Data<Pool<ConnectionManager<PgConnection>>>>() {
                // log_debug!("Connecting to database...");
                if let Some(authen_header) = req.headers().get("Authorization") {
                    // log_debug!("Parsing authorization header...");
                    if let Ok(authen_str) = authen_header.to_str() {
                        if authen_str.starts_with("bearer") || authen_str.starts_with("Bearer") {
                            // log_debug!("Parsing token...");
                            let token = authen_str[6..authen_str.len()].trim();
                            if let Ok(token_data) = UserToken::decode_token(token.to_string()) {
                                // log_debug!("Decoding token...");
                                match TokenModel::token_ckeck(&token_data, &mut pool.get().unwrap())
                                {
                                    Ok(user_name) => {
                                        authenticate_pass = true;
                                        match UserModel::get_user_by_username(
                                            &user_name,
                                            &mut pool.get().unwrap(),
                                        ) {
                                            Ok(user_info) => {
                                                let gid_list = GroupModel::get_groups_by_uid(
                                                    user_info.id.unwrap(),
                                                    &mut pool.get().unwrap(),
                                                )
                                                .unwrap_or_default();

                                                let sid_list = ServiceModel::get_sids_by_route(
                                                    &req.uri().path(),
                                                    &mut pool.get().unwrap(),
                                                )
                                                .unwrap_or_default();

                                                match AccessModel::get_max_permission(
                                                    &gid_list
                                                        .into_iter()
                                                        .map(|group_info| group_info.id)
                                                        .collect(),
                                                    &sid_list,
                                                    &mut pool.get().unwrap(),
                                                ) {
                                                    Ok(access_int) => match access_int {
                                                        2 => permit_pass = true,
                                                        1 => {
                                                            if Method::GET == req.method() {
                                                                permit_pass = true;
                                                            }
                                                        }
                                                        _ => (),
                                                    },
                                                    Err(e) => {
                                                        internal_error = (true, e.1.clone());
                                                    }
                                                };
                                                // log_debug!("find {:?}", req.uri());
                                            }
                                            Err(e) => {
                                                internal_error = (true, e.1.clone());
                                            }
                                        };
                                    }
                                    Err(_) => {
                                        // log_debug!("Valid token");
                                    }
                                }
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
                    "Invalid token",
                    Some("please login again".to_string()),
                    1,
                ))
                .map_into_right_body();

            return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
        }

        if !permit_pass {
            let (request, _pl) = req.into_parts();
            let response = HttpResponse::Unauthorized()
                .json(MailManErr::new(
                    403,
                    "Forbidden",
                    Some("User has no permissions on the resource".to_string()),
                    1,
                ))
                .map_into_right_body();

            return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
        }

        let res = self.service.call(req);

        Box::pin(async move { res.await.map(ServiceResponse::map_into_left_body) })
    }
}
