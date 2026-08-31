use actix_web::{HttpRequest, HttpResponse, http::Method, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use share_lib::data_structure::MailManErr;
use share_lib::err_mapping::MailManErrResponser;

use crate::service::auth_service;

// POST api/auth/verify
// 子系统回源鉴权接口：不面向终端用户，调用方须携带
// `Authorization: uuid <subsys_uuid>` 自证为已注册且启用的子系统
pub async fn verify(
    req: HttpRequest,
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    // 1. 调用方认证：必须是已注册且启用的子系统
    let subsys_uuid = match req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
    {
        Some(s) if s.starts_with("uuid ") => s[5..].trim(),
        _ => {
            return Err(MailManErrResponser::mapping_from_mme(MailManErr::new(
                401,
                "Unauthorized",
                Some("missing or invalid subsystem uuid header".to_string()),
                1,
            )));
        }
    };
    if let Err(err_mm) = auth_service::authenticate_subsys(subsys_uuid, &pool).await {
        return Err(MailManErrResponser::mapping_from_mme(err_mm));
    }

    // 2. 请求体清洗：token / path / method 均为必填字符串，method 须为合法 HTTP 方法
    let token = match map.get("token").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => {
            return Err(MailManErrResponser::mapping_from_mme(MailManErr::new(
                400,
                "Bad Request",
                Some("Missing or invalid `token` field.".to_string()),
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

    // 3. 回源鉴权
    match auth_service::verify(token, path, &method, &pool).await {
        Ok(verify_res) => Ok(HttpResponse::Ok().json(verify_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}
