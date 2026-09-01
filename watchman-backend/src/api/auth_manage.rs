use actix_web::{HttpMessage, HttpRequest, HttpResponse, http::Method, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use share_lib::data_structure::MailManErr;
use share_lib::err_mapping::MailManErrResponser;

use crate::service::auth_service;
use crate::service::auth_service::AuthIdentity;

// POST api/auth/verify
// 子系统回源鉴权接口：路由已挂入 /api scope，调用方把用户 JWT 直接作为
// `Authorization: Bearer <用户JWT>` 携带，用户认证由 JwtAuth 中间件完成
// （身份注入 extensions）；本 handler 负责 body 清洗、子系统 name/uuid
// 匹配校验，并对 body 内目标 path/method 做权限判定
pub async fn verify(
    req: HttpRequest,
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    // 1. 用户身份：由 JwtAuth 注入 extensions，取不到说明中间件链路异常
    let (uid, username) = {
        let ext = req.extensions();
        match ext.get::<AuthIdentity>() {
            Some(identity) => (identity.uid, identity.username.clone()),
            None => {
                return Err(MailManErrResponser::mapping_from_mme(MailManErr::new(
                    500,
                    "Internal Server Error",
                    Some("missing auth identity in request extensions".to_string()),
                    1,
                )));
            }
        }
    };

    // 2. 请求体清洗：subsys_name / subsys_uuid / path / method 均为必填字符串，
    //    method 须为合法 HTTP 方法
    let subsys_name = match map.get("subsys_name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => {
            return Err(MailManErrResponser::mapping_from_mme(MailManErr::new(
                400,
                "Bad Request",
                Some("Missing or invalid `subsys_name` field.".to_string()),
                1,
            )));
        }
    };
    let subsys_uuid = match map.get("subsys_uuid").and_then(|v| v.as_str()) {
        Some(u) => u,
        None => {
            return Err(MailManErrResponser::mapping_from_mme(MailManErr::new(
                400,
                "Bad Request",
                Some("Missing or invalid `subsys_uuid` field.".to_string()),
                1,
            )));
        }
    };
    let path = match map.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => {
            return Err(MailManErrResponser::mapping_from_mme(MailManErr::new(
                400,
                "Bad Request",
                Some("Missing or invalid `path` field.".to_string()),
                1,
            )));
        }
    };
    let method = match map
        .get("method")
        .and_then(|v| v.as_str())
        .and_then(|m| Method::from_bytes(m.as_bytes()).ok())
    {
        Some(m) => m,
        None => {
            return Err(MailManErrResponser::mapping_from_mme(MailManErr::new(
                400,
                "Bad Request",
                Some("Missing or invalid `method` field.".to_string()),
                1,
            )));
        }
    };

    // 3. 调用方子系统校验：subsys_name 须对应已注册且启用的子系统，且 uuid 匹配
    if let Err(err_mm) = auth_service::authenticate_subsys(subsys_name, subsys_uuid, &pool).await {
        return Err(MailManErrResponser::mapping_from_mme(err_mm));
    }

    // 4. 对 body 内目标资源做权限判定并组装结果
    match auth_service::verify(uid, username, path, &method, &pool).await {
        Ok(verify_res) => Ok(HttpResponse::Ok().json(verify_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}
