#![allow(unused_imports)]
use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use share_lib::err_mapping::MailManErrResponser;

use crate::{
    model::{
        notification_alias::{NewNotificationAlias, UpdateNotificationAlias},
        notification_template::{NewNotificationTemplate, UpdateNotificationTemplate},
    },
    services::channel::NotificationRequest,
    services::manage_service,
    services::notification_router::NotificationRouter,
    services::notify_service,
};

// ==================== 统一通知发送 API ====================

/// POST /api/notify/send - 统一发送接口
///
/// recipients 格式: [{"channel_type": "smtp", "recipient": "user@example.com"}, ...]
pub async fn send(
    req: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let title = req
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let body = req
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let format = req
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("text")
        .to_string();

    let priority = req
        .get("priority")
        .and_then(|v| v.as_str())
        .unwrap_or("normal")
        .to_string();

    let tags = req
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let url = req
        .get("url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mentions = req
        .get("mentions")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let request = NotificationRequest {
        title,
        body,
        format,
        priority,
        tags,
        url,
        mentions,
        template_id: None,
        params: serde_json::Value::Null,
    };

    let recipients = match notify_service::resolve_recipients(req.get("recipients"), &pool).await {
        Ok(recipients) => recipients,
        Err(err) => return Err(MailManErrResponser::mapping_from_mme(err)),
    };

    let router = NotificationRouter::new();

    match router.send_to_channels(request, recipients, &pool).await {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

/// POST /api/notify/template - 使用模板发送
pub async fn send_with_template(
    req: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let template_name = req
        .get("template_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
                400,
                "Bad Request",
                Some("Missing 'template_name' field".to_string()),
                1,
            ))
        })?;

    let variables = req
        .get("variables")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let recipients = match notify_service::resolve_recipients(req.get("recipients"), &pool).await {
        Ok(recipients) => recipients,
        Err(err) => return Err(MailManErrResponser::mapping_from_mme(err)),
    };

    let router = NotificationRouter::new();

    match router
        .send_with_template(template_name, variables, recipients, &pool)
        .await
    {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

// ==================== 模板管理 API ====================

/// GET /api/template/get - 获取模板列表
pub async fn get_templates(
    query: web::Query<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match manage_service::get_templates(query.into_inner(), &pool).await {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

/// POST /api/template/new - 创建模板
pub async fn create_template(
    req: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let name = req.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'name' field".to_string()),
            1,
        ))
    })?;

    let new_template = NewNotificationTemplate {
        name: name.to_string(),
        description: req
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        is_enabled: req.get("is_enabled").and_then(|v| v.as_bool()),
        params_template: req.get("params_template").cloned(),
        smtp: req.get("smtp").cloned(),
        bark: req.get("bark").cloned(),
        gotify: req.get("gotify").cloned(),
        ntfy: req.get("ntfy").cloned(),
        teams: req.get("teams").cloned(),
        webhook: req.get("webhook").cloned(),
    };

    match manage_service::create_template(new_template, &pool).await {
        Ok(id) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Template created",
                Some(format!("Template ID: {}", id)),
            )),
        ),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

/// POST /api/template/update - 更新模板
pub async fn update_template(
    req: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let template_id = req.get("id").and_then(|v| v.as_i64()).ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'id' field".to_string()),
            1,
        ))
    })? as i32;

    let update = UpdateNotificationTemplate {
        name: req
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        description: req
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        is_enabled: req.get("is_enabled").and_then(|v| v.as_bool()),
        params_template: req.get("params_template").cloned(),
        smtp: req.get("smtp").cloned(),
        bark: req.get("bark").cloned(),
        gotify: req.get("gotify").cloned(),
        ntfy: req.get("ntfy").cloned(),
        teams: req.get("teams").cloned(),
        webhook: req.get("webhook").cloned(),
        updated_at: Some(chrono::Local::now().naive_local()),
    };

    match manage_service::update_template(template_id, update, &pool).await {
        Ok(num) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Template updated",
                Some(format!("Rows affected: {}", num)),
            )),
        ),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

/// POST /api/template/delete - 删除模板
pub async fn delete_template(
    req: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let template_id = req.get("id").and_then(|v| v.as_i64()).ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'id' field".to_string()),
            1,
        ))
    })? as i32;

    match manage_service::delete_template(template_id, &pool).await {
        Ok(num) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Template deleted",
                Some(format!("Rows affected: {}", num)),
            )),
        ),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

// ==================== 通知记录查询 API ====================

/// GET /api/record/get - 获取通知记录
pub async fn get_records(
    query: web::Query<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match manage_service::get_records(query.into_inner(), &pool).await {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

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

// ==================== Alias 管理 API ====================

/// GET /api/alias/get - 获取所有 alias
pub async fn get_aliases(
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match manage_service::get_aliases(&pool).await {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

/// POST /api/alias/new - 创建 alias
pub async fn create_alias(
    req: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let name = req.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'name' field".to_string()),
            1,
        ))
    })?;

    let recipients = req.get("recipients").cloned().ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'recipients' field".to_string()),
            1,
        ))
    })?;

    let new_alias = NewNotificationAlias {
        name: name.to_string(),
        description: req
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        recipients,
        is_enabled: req.get("is_enabled").and_then(|v| v.as_bool()),
    };

    match manage_service::create_alias(new_alias, &pool).await {
        Ok(id) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Alias created",
                Some(format!("Alias ID: {}", id)),
            )),
        ),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

/// POST /api/alias/update - 更新 alias
pub async fn update_alias(
    req: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let alias_id = req.get("id").and_then(|v| v.as_i64()).ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'id' field".to_string()),
            1,
        ))
    })? as i32;

    let update = UpdateNotificationAlias {
        name: req
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        description: req
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        recipients: req.get("recipients").cloned(),
        is_enabled: req.get("is_enabled").and_then(|v| v.as_bool()),
        updated_at: Some(chrono::Local::now().naive_local()),
    };

    match manage_service::update_alias(alias_id, update, &pool).await {
        Ok(num) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Alias updated",
                Some(format!("Rows affected: {}", num)),
            )),
        ),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

/// POST /api/alias/delete - 删除 alias
pub async fn delete_alias(
    req: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let alias_id = req.get("id").and_then(|v| v.as_i64()).ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'id' field".to_string()),
            1,
        ))
    })? as i32;

    match manage_service::delete_alias(alias_id, &pool).await {
        Ok(num) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Alias deleted",
                Some(format!("Rows affected: {}", num)),
            )),
        ),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}
