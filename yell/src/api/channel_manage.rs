use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use share_lib::err_mapping::MailManErrResponser;

use crate::services::manage_service;

// ==================== 通道配置 API ====================

/// GET /api/channel/get - 获取通道配置
pub async fn get_channel_configs(
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match manage_service::get_channel_configs(&pool).await {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

/// POST /api/channel/update - 更新通道配置
pub async fn update_channel_config(
    req: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let channel_type = req
        .get("channel_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
                400,
                "Bad Request",
                Some("Missing 'channel_type' field".to_string()),
                1,
            ))
        })?;

    let config_json = req.get("config_json").cloned().ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'config_json' field".to_string()),
            1,
        ))
    })?;

    let is_enabled = req
        .get("is_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let instance_name = req.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'name' field (instance name is required)".to_string()),
            1,
        ))
    })?;

    match manage_service::update_channel_config(
        channel_type.to_string(),
        instance_name.to_string(),
        config_json,
        is_enabled,
        &pool,
    )
    .await
    {
        Ok(num) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Channel config updated",
                Some(format!("Rows affected: {}", num)),
            )),
        ),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}
