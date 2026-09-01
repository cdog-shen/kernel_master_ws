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

use crate::{server::GLOBAL_CONFIG, service::auth_service};

/// 把 MailManErr 按 code 映射为 HTTP 错误响应（中间件短路返回用）
fn mme_into_response<B>(
    req: ServiceRequest,
    err: MailManErr<String>,
) -> ServiceResponse<EitherBody<B>> {
    let (req, _) = req.into_parts();
    let resp = match err.code {
        400 => HttpResponse::BadRequest().json(err),
        401 => HttpResponse::Unauthorized().json(err),
        403 => HttpResponse::Forbidden().json(err),
        _ => HttpResponse::InternalServerError().json(err),
    }
    .map_into_right_body();
    ServiceResponse::new(req, resp)
}

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
            // 取 Bearer token 并调共用认证逻辑
            let auth_res = {
                let hdr = match req.headers().get("authorization") {
                    Some(h) => h,
                    None => {
                        return Ok(mme_into_response(
                            req,
                            MailManErr::new(
                                401,
                                "Unauthorized",
                                Some("missing authorization header".into()),
                                1,
                            ),
                        ));
                    }
                };
                let tok = match hdr.to_str() {
                    Ok(s) if s.starts_with("Bearer ") => &s[7..],
                    _ => {
                        return Ok(mme_into_response(
                            req,
                            MailManErr::new(
                                401,
                                "Unauthorized",
                                Some("invalid bearer format".into()),
                                1,
                            ),
                        ));
                    }
                };
                auth_service::authenticate(tok, &pool).await
            };

            let identity = match auth_res {
                Ok(i) => i,
                Err(e) => return Ok(mme_into_response(req, e)),
            };

            // 把身份塞进 extensions，下游按需读取：
            // PermissionCheck 取 uid（i32），/api/auth/verify 的 handler 取完整 AuthIdentity
            req.extensions_mut().insert(identity.uid);
            req.extensions_mut().insert(identity);

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
            // 调共用权限判定逻辑
            match auth_service::authorize(uid, &path, &method, &pool).await {
                Ok(_) => svc.call(req).await.map(ServiceResponse::map_into_left_body),
                Err(e) => Ok(mme_into_response(req, e)),
            }
        })
    }
}
