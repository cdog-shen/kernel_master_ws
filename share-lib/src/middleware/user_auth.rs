//! 子系统统一用户鉴权中间件
//!
//! 认证链路（按 `Authorization` header 的 scheme 区分，大小写不敏感）：
//! - `Bearer <jwt>`：用户直连新链路。把用户 JWT 原样作为 `Authorization` 转发给 watchman
//!   `POST /api/auth/verify` 回源鉴权（body 携带 `subsys_name`/`subsys_uuid`/`path`/`method`），
//!   通过后把 `uid`（i32）写入 request extensions 放行；回源 401/403 原样映射，
//!   网络失败/超时按 503 处理（需 feature `http`，individual 模式下整段编译期裁掉）
//! - `uuid <uuid>`：watchman 转发旧链路。与本机 `subsys_uuid` 比对，一致放行
//!   （过渡期保留，旧链路下线时删除该分支）
//!
//! 错误响应与各 crate 原中间件 wire 格式一致：body 为 `MailManErr` 序列化。

use std::rc::Rc;
use std::sync::Arc;

use actix_service::forward_ready;
#[cfg(all(feature = "http", not(feature = "individual")))]
use actix_web::HttpMessage;
use actix_web::{
    Error, HttpResponse,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::Method,
};
use futures::future::{LocalBoxFuture, Ready, ok};

use crate::data_structure::MailManErr;

// 非 individual 模式下回源鉴权依赖 http_client（feature `http`），缺了直接拒绝编译
#[cfg(all(not(feature = "http"), not(feature = "individual")))]
compile_error!(
    "user_auth 中间件在非 individual 模式下依赖 feature `http`（回源鉴权），请开启 `http` 或改用 `individual`"
);

/// 中间件配置（share-lib 不持有各 crate 的 GLOBAL_CONFIG，由调用方构造时注入）
pub struct UserAuthConfig {
    /// watchman 地址（回源用，individual 模式下裁掉）
    #[cfg(not(feature = "individual"))]
    pub master_addr: String,
    /// watchman 端口（回源用，individual 模式下裁掉）
    #[cfg(not(feature = "individual"))]
    pub master_port: u16,
    /// 本机子系统名称（回源自证用，individual 模式下裁掉）
    #[cfg(not(feature = "individual"))]
    pub subsys_name: String,
    /// 本机子系统 uuid（旧链路校验 + 回源自证）
    pub subsys_uuid: String,
    /// 认证白名单：path 前缀命中即放行
    pub authenticate_bypass: Vec<String>,
}

/// 把 MailManErr 按 code 映射为 HTTP 错误响应（中间件短路返回用）
fn mme_into_response<B>(
    req: ServiceRequest,
    err: MailManErr<'static, String>,
) -> ServiceResponse<EitherBody<B>> {
    let (req, _) = req.into_parts();
    let resp = match err.code {
        400 => HttpResponse::BadRequest().json(err),
        401 => HttpResponse::Unauthorized().json(err),
        403 => HttpResponse::Forbidden().json(err),
        503 => HttpResponse::ServiceUnavailable().json(err),
        _ => HttpResponse::InternalServerError().json(err),
    }
    .map_into_right_body();
    ServiceResponse::new(req, resp)
}

// ---------- UserAuth ----------

/// 子系统统一鉴权中间件（Transform 入口）
pub struct UserAuth {
    config: Arc<UserAuthConfig>,
}

impl UserAuth {
    pub fn new(config: UserAuthConfig) -> Self {
        UserAuth {
            config: Arc::new(config),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for UserAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = UserAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(UserAuthMiddleware {
            service: Rc::new(service),
            config: self.config.clone(),
        })
    }
}

/// UserAuth 的 Service 实现
pub struct UserAuthMiddleware<S> {
    service: Rc<S>,
    config: Arc<UserAuthConfig>,
}

impl<S, B> Service<ServiceRequest> for UserAuthMiddleware<S>
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
        let config = self.config.clone();

        // 1. OPTIONS 与白名单前缀直接放行
        let bypassed = *req.method() == Method::OPTIONS
            || config
                .authenticate_bypass
                .iter()
                .any(|prefix| req.path().starts_with(prefix));
        if bypassed {
            return Box::pin(async move {
                svc.call(req).await.map(ServiceResponse::map_into_left_body)
            });
        }

        // 2. 解析 Authorization header（拆为 scheme 与凭证两段）
        let header = req
            .headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split_once(' '))
            .map(|(scheme, cred)| (scheme.to_ascii_lowercase(), cred.trim().to_owned()));
        let (scheme, credential) = match header {
            Some(parts) => parts,
            None => {
                return Box::pin(async move {
                    Ok(mme_into_response(
                        req,
                        MailManErr::new(
                            401,
                            "Unauthorized",
                            Some("missing or invalid authorization header".to_string()),
                            1,
                        ),
                    ))
                });
            }
        };

        // 3. uuid 旧链路：与本机 subsys_uuid 比对（过渡期保留）
        if scheme == "uuid" {
            if credential == config.subsys_uuid {
                return Box::pin(async move {
                    svc.call(req).await.map(ServiceResponse::map_into_left_body)
                });
            }
            return Box::pin(async move {
                Ok(mme_into_response(
                    req,
                    MailManErr::new(
                        401,
                        "Unauthorized",
                        Some("subsystem uuid mismatch".to_string()),
                        1,
                    ),
                ))
            });
        }

        // 4. Bearer 新链路：回源 watchman /api/auth/verify（individual 模式裁掉）
        #[cfg(all(feature = "http", not(feature = "individual")))]
        if scheme == "bearer" {
            let path = req.path().to_owned();
            let method = req.method().clone();
            let url = format!(
                "http://{}:{}/api/auth/verify",
                config.master_addr, config.master_port
            );
            // 回源：把用户的 JWT 原样作为 Authorization 转发给 watchman（由其 JwtAuth 验签）
            let headers = vec![("authorization".to_owned(), format!("Bearer {credential}"))];
            // 新约定 body：子系统自证（name + uuid）+ 目标资源（path + method），
            // watchman 校验 name/uuid 匹配后做权限判定
            let body = serde_json::json!({
                "subsys_name": config.subsys_name,
                "subsys_uuid": config.subsys_uuid,
                "path": path,
                "method": method.as_str(),
            });

            return Box::pin(async move {
                // ureq 为同步阻塞实现，回源调用丢到 blocking 线程池，避免卡住 actix worker
                let verify_res = actix_web::web::block(move || {
                    crate::infrastructure::http_client::post_json_with_status(
                        &url,
                        &headers,
                        &[],
                        &body,
                    )
                })
                .await;

                // 回源结果清洗：2xx 取响应体；401/403 原样映射；其余非 2xx 与网络错误按 503
                let resp_text = match verify_res {
                    Ok(Ok((status, text))) => match status {
                        200..=299 => text,
                        401 => {
                            return Ok(mme_into_response(
                                req,
                                MailManErr::new(
                                    401,
                                    "Unauthorized",
                                    Some("token invalid or expired".to_string()),
                                    1,
                                ),
                            ));
                        }
                        403 => {
                            return Ok(mme_into_response(
                                req,
                                MailManErr::new(
                                    403,
                                    "Forbidden",
                                    Some("permission denied by master".to_string()),
                                    1,
                                ),
                            ));
                        }
                        _ => {
                            return Ok(mme_into_response(
                                req,
                                MailManErr::new(
                                    503,
                                    "Service Unavailable",
                                    Some(format!(
                                        "master verify unavailable: upstream status {status}"
                                    )),
                                    1,
                                ),
                            ));
                        }
                    },
                    Ok(Err(mme)) => {
                        return Ok(mme_into_response(
                            req,
                            MailManErr::new(
                                503,
                                "Service Unavailable",
                                Some(format!(
                                    "master verify unavailable: {}",
                                    mme.msg.unwrap_or_default()
                                )),
                                1,
                            ),
                        ));
                    }
                    Err(e) => {
                        return Ok(mme_into_response(
                            req,
                            MailManErr::new(
                                503,
                                "Service Unavailable",
                                Some(format!("master verify worker pool error: {e}")),
                                1,
                            ),
                        ));
                    }
                };

                // 解析 MailManOk 响应体：{"code":200,"data":{"uid":1,"permission":2,...}}
                let data = serde_json::from_str::<serde_json::Value>(&resp_text)
                    .ok()
                    .and_then(|v| v.get("data").cloned());
                let pair = data.as_ref().map(|d| {
                    (
                        d.get("uid").and_then(|v| v.as_i64()).map(|v| v as i32),
                        d.get("permission").and_then(|v| v.as_i64()),
                    )
                });
                let (uid, permission) = match pair {
                    Some((Some(uid), Some(perm))) => (uid, perm),
                    _ => {
                        return Ok(mme_into_response(
                            req,
                            MailManErr::new(
                                503,
                                "Service Unavailable",
                                Some("master verify response malformed".to_string()),
                                1,
                            ),
                        ));
                    }
                };

                // 权限语义：2=读写放行；1=只读仅 GET 放行；其余拒绝
                let permitted = permission == 2 || (permission == 1 && method == Method::GET);
                if !permitted {
                    return Ok(mme_into_response(
                        req,
                        MailManErr::new(
                            403,
                            "Forbidden",
                            Some("read-only permission, write denied".to_string()),
                            1,
                        ),
                    ));
                }

                // 把 uid 塞进 extensions，下游 handler 可取
                req.extensions_mut().insert(uid);

                svc.call(req).await.map(ServiceResponse::map_into_left_body)
            });
        }

        // 5. 其余 scheme（含 individual 模式下的 Bearer）一律 401
        Box::pin(async move {
            Ok(mme_into_response(
                req,
                MailManErr::new(
                    401,
                    "Unauthorized",
                    Some("unsupported authorization scheme".to_string()),
                    1,
                ),
            ))
        })
    }
}
