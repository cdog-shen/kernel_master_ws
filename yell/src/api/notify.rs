#![allow(unused_imports)]
use actix_web::{HttpResponse, web};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use serde_json::{Map, Value};

use crate::{
    model::{
        channel_config::ChannelConfig,
        notification_record::NotificationRecord,
        notification_template::{
            NewNotificationTemplate, NotificationTemplate, UpdateNotificationTemplate,
        },
    },
    services::channel::NotificationRequest,
    services::notification_router::NotificationRouter,
    util::err_mapping::MailManErrResponser,
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
    };

    // 解析 recipients: [{"channel_type": "...", "recipient": "..."}, ...]
    let recipients = req
        .get("recipients")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
                400,
                "Bad Request",
                Some("Missing 'recipients' field".to_string()),
                1,
            ))
        })?
        .iter()
        .filter_map(|v| {
            if let Some(obj) = v.as_object() {
                let channel_type = obj.get("channel_type")?.as_str()?.to_string();
                let recipient = obj.get("recipient")?.as_str()?.to_string();
                Some((channel_type, recipient))
            } else {
                None
            }
        })
        .collect();

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

    let recipients = req
        .get("recipients")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    if let Some(obj) = v.as_object() {
                        let channel_type = obj.get("channel_type")?.as_str()?.to_string();
                        let recipient = obj.get("recipient")?.as_str()?.to_string();
                        Some((channel_type, recipient))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

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
    match web::block({
        let pool = pool.clone();
        let filter = query.into_inner();
        move || {
            let mut conn = pool.get().unwrap();
            NotificationTemplate::get_with_filter(&filter, &mut conn)
        }
    })
    .await
    {
        Ok(Ok(data)) => Ok(HttpResponse::Ok().json(data)),
        Ok(Err((_, msg))) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to get templates", Some(msg), 0),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to get templates", Some(e.to_string()), 0),
        )),
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

    let channel_type = req
        .get("channel_type")
        .and_then(|v| v.as_str())
        .unwrap_or("smtp")
        .to_string();

    let content_template = req
        .get("content_template")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
                400,
                "Bad Request",
                Some("Missing 'content_template' field".to_string()),
                1,
            ))
        })?;

    let new_template = NewNotificationTemplate {
        name: name.to_string(),
        description: req.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
        channel_type,
        subject_template: req.get("subject_template").and_then(|v| v.as_str()).map(|s| s.to_string()),
        content_template: content_template.to_string(),
        content_format: req.get("content_format").and_then(|v| v.as_str()).map(|s| s.to_string()),
        is_enabled: req.get("is_enabled").and_then(|v| v.as_bool()),
    };

    match web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().unwrap();
            NotificationTemplate::create(&new_template, &mut conn)
        }
    })
    .await
    {
        Ok(Ok(id)) => Ok(HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
            200,
            "Template created",
            Some(format!("Template ID: {}", id)),
        ))),
        Ok(Err((_, msg))) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to create template", Some(msg), 0),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to create template", Some(e.to_string()), 0),
        )),
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
        name: req.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        description: req.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
        channel_type: req.get("channel_type").and_then(|v| v.as_str()).map(|s| s.to_string()),
        subject_template: req.get("subject_template").and_then(|v| v.as_str()).map(|s| s.to_string()),
        content_template: req.get("content_template").and_then(|v| v.as_str()).map(|s| s.to_string()),
        content_format: req.get("content_format").and_then(|v| v.as_str()).map(|s| s.to_string()),
        is_enabled: req.get("is_enabled").and_then(|v| v.as_bool()),
        updated_at: Some(chrono::Local::now().naive_local()),
    };

    match web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().unwrap();
            NotificationTemplate::update(template_id, &update, &mut conn)
        }
    })
    .await
    {
        Ok(Ok(num)) => Ok(HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
            200,
            "Template updated",
            Some(format!("Rows affected: {}", num)),
        ))),
        Ok(Err((_, msg))) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to update template", Some(msg), 0),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to update template", Some(e.to_string()), 0),
        )),
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

    match web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().unwrap();
            NotificationTemplate::delete(template_id, &mut conn)
        }
    })
    .await
    {
        Ok(Ok(num)) => Ok(HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
            200,
            "Template deleted",
            Some(format!("Rows affected: {}", num)),
        ))),
        Ok(Err((_, msg))) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to delete template", Some(msg), 0),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to delete template", Some(e.to_string()), 0),
        )),
    }
}

// ==================== 通知记录查询 API ====================

/// GET /api/record/get - 获取通知记录
pub async fn get_records(
    query: web::Query<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match web::block({
        let pool = pool.clone();
        let filter = query.into_inner();
        move || {
            let mut conn = pool.get().unwrap();
            NotificationRecord::get_with_filter(&filter, &mut conn)
        }
    })
    .await
    {
        Ok(Ok(data)) => Ok(HttpResponse::Ok().json(data)),
        Ok(Err((_, msg))) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to get records", Some(msg), 0),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to get records", Some(e.to_string()), 0),
        )),
    }
}

// ==================== 通道配置 API ====================

/// GET /api/channel/get - 获取通道配置
pub async fn get_channel_configs(
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().unwrap();
            ChannelConfig::get_all(&mut conn)
        }
    })
    .await
    {
        Ok(Ok(data)) => Ok(HttpResponse::Ok().json(data)),
        Ok(Err((_, msg))) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to get channel configs", Some(msg), 0),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to get channel configs", Some(e.to_string()), 0),
        )),
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

    let is_enabled = req.get("is_enabled").and_then(|v| v.as_bool()).unwrap_or(true);

    match web::block({
        let pool = pool.clone();
        let channel = channel_type.to_string();
        move || {
            let mut conn = pool.get().unwrap();
            ChannelConfig::upsert(&channel, &config_json, is_enabled, &mut conn)
        }
    })
    .await
    {
        Ok(Ok(num)) => Ok(HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
            200,
            "Channel config updated",
            Some(format!("Rows affected: {}", num)),
        ))),
        Ok(Err((_, msg))) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to update channel config", Some(msg), 0),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to update channel config", Some(e.to_string()), 0),
        )),
    }
}
