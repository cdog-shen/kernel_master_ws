use actix_web::{HttpResponse, web};
use diesel::{PgConnection, r2d2::{ConnectionManager, Pool}};
use serde_json::{Map, Value};
use share_lib::data_structure::MailManErr;

use crate::{model::webhook, service::webhook_service, util::err_mapping::MailManErrResponser};

// GET api/webhook
pub async fn all_webhook(
    query: web::Query<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match webhook_service::all_webhook(query.0, &pool) {
        Ok(webhook_data) => Ok(HttpResponse::Ok().json(webhook_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/webhook
pub async fn new_webhook(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let webhook_info = webhook::WebhookInfo::from_map(map.0).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
    })?;

    let webhook_info = webhook::WebhookInfo {
        id: None,
        hook_name: Some(webhook_info.hook_name.clone().ok_or(
            MailManErrResponser::mapping_from_mme(MailManErr::new(
                400,
                "Bad Request",
                Some("Missing `hook_name` field.".to_string()),
                1,
            )),
        )?),
        target_url: Some(
            webhook_info
                .target_url
                .clone()
                .ok_or(MailManErrResponser::mapping_from_mme(MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing `target_url` field.".to_string()),
                    1,
                )))?,
        ),
        method_type: Some(webhook_info.method_type.clone().unwrap_or("POST".to_string())),
        token: None, // token将在service层自动生成
        header_json: webhook_info.header_json,
        body_json: webhook_info.body_json,
        query_json: webhook_info.query_json,
        ttl: Some(webhook_info.ttl.unwrap_or(86400)), // 默认24小时
        is_enable: Some(webhook_info.is_enable.unwrap_or(true)),
        update_time: webhook_info.update_time,
    };

    match webhook_service::new_webhook(webhook_info, &pool) {
        Ok(webhook_data) => Ok(HttpResponse::Ok().json(webhook_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// PATCH api/webhook
pub async fn update_webhook(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let webhook_info = webhook::WebhookInfo::from_map(map.0).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
    })?;

    // 确保id字段存在
    if webhook_info.id.is_none() {
        return Err(MailManErrResponser::mapping_from_mme(MailManErr::new(
            400,
            "Bad Request",
            Some("Missing `id` field for update.".to_string()),
            1,
        )));
    }

    match webhook_service::update_webhook(webhook_info, &pool) {
        Ok(webhook_data) => Ok(HttpResponse::Ok().json(webhook_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// DELETE api/webhook
pub async fn delete_webhook(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let id = map
        .0
        .get("id")
        .ok_or_else(|| {
            MailManErrResponser::mapping_from_mme(MailManErr::new(
                400,
                "Bad Request",
                Some("Can Not find 'id' field".to_string()),
                1,
            ))
        })?
        .as_i64()
        .ok_or_else(|| {
            MailManErrResponser::mapping_from_mme(MailManErr::new(
                400,
                "Bad Request",
                Some("'id' field MUST be i32".to_string()),
                1,
            ))
        })? as i32;

    match webhook_service::delete_webhook(id, &pool) {
        Ok(webhook_data) => Ok(HttpResponse::Ok().json(webhook_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/webhook/{hook_name}
pub async fn post_webhook(
    hook_name: web::Path<String>,
    _params: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match webhook_service::post_webhook(hook_name.to_string(), &pool) {
        Ok(webhook_data) => Ok(HttpResponse::Ok().json(webhook_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// GET api/webhook/{hook_name}
pub async fn get_webhook(
    hook_name: web::Path<String>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match webhook_service::get_webhook(hook_name.to_string(), &pool) {
        Ok(webhook_data) => Ok(HttpResponse::Ok().json(webhook_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}