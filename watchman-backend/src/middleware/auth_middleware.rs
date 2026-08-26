use std::rc::Rc;

use actix_service::forward_ready;
use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::Method,
    web::Data,
};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use futures::future::{LocalBoxFuture, Ready, ok};

use share_lib::data_structure::MailManErr;

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

// ---------- JWTAuth ----------
pub struct JwtAuth;

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtAuthMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        // 白名单
        let bypass = GLOBAL_CONFIG
            .read()
            .unwrap()
            .authenticate_bypass
            .iter()
            .any(|p| req.path().starts_with(p));
        if bypass || *req.method() == Method::OPTIONS {
            return Box::pin(async move {
                svc.call(req).await.map(ServiceResponse::map_into_left_body)
            });
        }

        let pool = match req.app_data::<Data<Pool<ConnectionManager<PgConnection>>>>() {
            Some(p) => p.clone(),
            None => {
                let (req, _) = req.into_parts();
                let resp = HttpResponse::InternalServerError()
                    .json(MailManErr::<String>::new(
                        500,
                        "Internal Server Error",
                        Some("missing db pool".into()),
                        1,
                    ))
                    .map_into_right_body();
                return Box::pin(async move { Ok(ServiceResponse::new(req, resp)) });
            }
        };

        Box::pin(async move {
            let uid = {
                let hdr = match req.headers().get("authorization") {
                    Some(h) => h,
                    None => {
                        let (req, _) = req.into_parts();
                        let resp = HttpResponse::Unauthorized()
                            .json(MailManErr::<String>::new(
                                401,
                                "Unauthorized",
                                Some("missing authorization header".into()),
                                1,
                            ))
                            .map_into_right_body();
                        return Ok(ServiceResponse::new(req, resp));
                    }
                };
                let tok = match hdr.to_str() {
                    Ok(s) if s.starts_with("Bearer ") => &s[7..],
                    _ => {
                        let (req, _) = req.into_parts();
                        let resp = HttpResponse::Unauthorized()
                            .json(MailManErr::<String>::new(
                                401,
                                "Unauthorized",
                                Some("invalid bearer format".into()),
                                1,
                            ))
                            .map_into_right_body();
                        return Ok(ServiceResponse::new(req, resp));
                    }
                };
                let claims = match UserToken::decode_token(tok.to_string()) {
                    Ok(c) => c,
                    Err(_) => {
                        let (req, _) = req.into_parts();
                        let resp = HttpResponse::Unauthorized()
                            .json(MailManErr::<String>::new(
                                401,
                                "Unauthorized",
                                Some("invalid or expired token".into()),
                                1,
                            ))
                            .map_into_right_body();
                        return Ok(ServiceResponse::new(req, resp));
                    }
                };
                let mut conn = pool.get().unwrap();
                let username = match TokenModel::token_ckeck(&claims, &mut conn) {
                    Ok(u) => u,
                    Err(_) => {
                        let (req, _) = req.into_parts();
                        let resp = HttpResponse::Unauthorized()
                            .json(MailManErr::<String>::new(
                                401,
                                "Unauthorized",
                                Some("token not found in store".into()),
                                1,
                            ))
                            .map_into_right_body();
                        return Ok(ServiceResponse::new(req, resp));
                    }
                };
                let user = UserModel::get_user_by_username(&username, &mut conn).unwrap();
                user.id.unwrap()
            };

            // 把 uid 塞进 extensions，下游中间件或 handler 都能拿到
            req.extensions_mut().insert(uid);

            svc.call(req).await.map(ServiceResponse::map_into_left_body)
        })
    }
}

// ---------- PermissionCheck ----------
pub struct PermissionCheck;

impl<S, B> Transform<S, ServiceRequest> for PermissionCheck
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = PermissionCheckMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(PermissionCheckMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct PermissionCheckMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for PermissionCheckMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        // 白名单
        let bypass = GLOBAL_CONFIG
            .read()
            .unwrap()
            .permit_bypass
            .iter()
            .any(|p| req.path().starts_with(p));
        if bypass || *req.method() == Method::OPTIONS {
            return Box::pin(async move {
                svc.call(req).await.map(ServiceResponse::map_into_left_body)
            });
        }

        let pool = match req.app_data::<Data<Pool<ConnectionManager<PgConnection>>>>() {
            Some(p) => p.clone(),
            None => {
                let (req, _) = req.into_parts();
                let resp = HttpResponse::InternalServerError()
                    .json(MailManErr::<String>::new(
                        500,
                        "Internal Server Error",
                        Some("missing db pool".into()),
                        1,
                    ))
                    .map_into_right_body();
                return Box::pin(async move { Ok(ServiceResponse::new(req, resp)) });
            }
        };

        let uid_opt = req.extensions().get::<i32>().copied();

        let uid = match uid_opt {
            Some(id) => id,
            None => {
                // 2. 此时已经没有借用，可以安全 move
                let (req, _) = req.into_parts();
                let resp = HttpResponse::Unauthorized()
                    .json(MailManErr::<String>::new(
                        401,
                        "Unauthorized",
                        Some("missing uid in extensions".into()),
                        1,
                    ))
                    .map_into_right_body();
                return Box::pin(async move { Ok(ServiceResponse::new(req, resp)) });
            }
        };

        let method = req.method().clone();
        let path = req.uri().path().to_string();

        Box::pin(async move {
            let mut conn = pool.get().unwrap();
            let gids = GroupModel::get_groups_by_uid(uid, &mut conn).unwrap_or_default();
            let gid_vec: Vec<i32> = gids.into_iter().map(|g| g.id).collect();
            let sids = ServiceModel::get_sids_by_route(&path, &mut conn).unwrap_or_default();

            let perm = match AccessModel::get_max_permission(&gid_vec, &sids, &mut conn) {
                Ok(p) => p,
                Err(e) => {
                    let (req, _) = req.into_parts();
                    let resp = HttpResponse::InternalServerError()
                        .json(MailManErr::new(500, "Internal Server Error", Some(e.1), 1))
                        .map_into_right_body();
                    return Ok(ServiceResponse::new(req, resp));
                }
            };

            let allowed = match perm {
                2 => true,
                1 => method == Method::GET,
                _ => false,
            };

            if !allowed {
                let (req, _) = req.into_parts();
                let resp = HttpResponse::Forbidden()
                    .json(MailManErr::<String>::new(
                        403,
                        "Forbidden",
                        Some("User has no permissions on the resource".into()),
                        1,
                    ))
                    .map_into_right_body();
                return Ok(ServiceResponse::new(req, resp));
            }

            svc.call(req).await.map(ServiceResponse::map_into_left_body)
        })
    }
}
