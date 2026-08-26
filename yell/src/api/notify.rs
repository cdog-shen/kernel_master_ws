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
        channel_config::ChannelConfig,
        notification_alias::NotificationAlias,
        notification_record::NotificationRecord,
        notification_template::{
            NewNotificationTemplate, NotificationTemplate, UpdateNotificationTemplate,
        },
    },
    services::channel::NotificationRequest,
    services::notification_router::NotificationRouter,
};

// ==================== recipients 解析辅助函数 ====================

/// 解析 recipients 字段，支持 String（alias）和 Array 两种格式
async fn resolve_recipients(
    recipients_value: Option<&Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<Vec<(String, String, String)>, MailManErrResponser> {
    match recipients_value {
        Some(Value::String(alias_name)) => {
            // String 格式：查找 alias
            let alias_name = alias_name.clone();
            let result = web::block({
                let pool = pool.clone();
                let name = alias_name.clone();
                move || {
                    let mut conn = pool.get().unwrap();
                    NotificationAlias::get_by_name(&name, &mut conn)
                }
            })
            .await;

            match result {
                Ok(Ok(Some(alias))) => {
                    let arr = alias.recipients.as_array().ok_or_else(|| {
                        MailManErrResponser::mapping_from_mme(
                            share_lib::data_structure::MailManErr::new(
                                500,
                                "Internal Error",
                                Some("Alias recipients is not an array".to_string()),
                                0,
                            ),
                        )
                    })?;
                    parse_recipients_array(arr)
                }
                Ok(Ok(None)) => Err(MailManErrResponser::mapping_from_mme(
                    share_lib::data_structure::MailManErr::new(
                        404,
                        "Not Found",
                        Some(format!("Alias '{}' not found or disabled", alias_name)),
                        1,
                    ),
                )),
                Ok(Err((_, msg))) => Err(MailManErrResponser::mapping_from_mme(
                    share_lib::data_structure::MailManErr::new(
                        500,
                        "Failed to get alias",
                        Some(msg),
                        0,
                    ),
                )),
                Err(e) => Err(MailManErrResponser::mapping_from_mme(
                    share_lib::data_structure::MailManErr::new(
                        500,
                        "Failed to get alias",
                        Some(e.to_string()),
                        0,
                    ),
                )),
            }
        }
        Some(Value::Array(arr)) => parse_recipients_array(arr),
        _ => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(
                400,
                "Bad Request",
                Some(
                    "Missing 'recipients' field (must be an array or alias name string)"
                        .to_string(),
                ),
                1,
            ),
        )),
    }
}

/// 解析 recipients JSON 数组为 Vec<(channel_type, recipient, instance)>
fn parse_recipients_array(
    arr: &[Value],
) -> Result<Vec<(String, String, String)>, MailManErrResponser> {
    let mut recipients = Vec::new();
    for v in arr {
        let obj = v.as_object().ok_or_else(|| {
            MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
                400,
                "Bad Request",
                Some("Each recipient must be an object".to_string()),
                1,
            ))
        })?;
        let channel_type = obj
            .get("channel_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing 'channel_type' in recipient".to_string()),
                    1,
                ))
            })?;
        let recipient = obj
            .get("recipient")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing 'recipient' in recipient".to_string()),
                    1,
                ))
            })?;
        let instance = obj
            .get("instance")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing 'instance' in recipient".to_string()),
                    1,
                ))
            })?;
        recipients.push((
            channel_type.to_string(),
            recipient.to_string(),
            instance.to_string(),
        ));
    }
    Ok(recipients)
}

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

    let recipients = resolve_recipients(req.get("recipients"), &pool).await?;

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

    let recipients = resolve_recipients(req.get("recipients"), &pool).await?;

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
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to get templates",
                Some(msg),
                0,
            ),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to get templates",
                Some(e.to_string()),
                0,
            ),
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

    match web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().unwrap();
            NotificationTemplate::create(&new_template, &mut conn)
        }
    })
    .await
    {
        Ok(Ok(id)) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Template created",
                Some(format!("Template ID: {}", id)),
            )),
        ),
        Ok(Err((_, msg))) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to create template",
                Some(msg),
                0,
            ),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to create template",
                Some(e.to_string()),
                0,
            ),
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

    match web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().unwrap();
            NotificationTemplate::update(template_id, &update, &mut conn)
        }
    })
    .await
    {
        Ok(Ok(num)) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Template updated",
                Some(format!("Rows affected: {}", num)),
            )),
        ),
        Ok(Err((_, msg))) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to update template",
                Some(msg),
                0,
            ),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to update template",
                Some(e.to_string()),
                0,
            ),
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
        Ok(Ok(num)) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Template deleted",
                Some(format!("Rows affected: {}", num)),
            )),
        ),
        Ok(Err((_, msg))) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to delete template",
                Some(msg),
                0,
            ),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to delete template",
                Some(e.to_string()),
                0,
            ),
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
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to get records",
                Some(e.to_string()),
                0,
            ),
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
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to get channel configs",
                Some(msg),
                0,
            ),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to get channel configs",
                Some(e.to_string()),
                0,
            ),
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

    match web::block({
        let pool = pool.clone();
        let channel = channel_type.to_string();
        let name = instance_name.to_string();
        move || {
            let mut conn = pool.get().unwrap();
            ChannelConfig::upsert(&channel, &name, &config_json, is_enabled, &mut conn)
        }
    })
    .await
    {
        Ok(Ok(num)) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Channel config updated",
                Some(format!("Rows affected: {}", num)),
            )),
        ),
        Ok(Err((_, msg))) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to update channel config",
                Some(msg),
                0,
            ),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to update channel config",
                Some(e.to_string()),
                0,
            ),
        )),
    }
}

// ==================== Alias 管理 API ====================

/// GET /api/alias/get - 获取所有 alias
pub async fn get_aliases(
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().unwrap();
            NotificationAlias::get_all(&mut conn)
        }
    })
    .await
    {
        Ok(Ok(data)) => Ok(HttpResponse::Ok().json(data)),
        Ok(Err((_, msg))) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to get aliases", Some(msg), 0),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to get aliases",
                Some(e.to_string()),
                0,
            ),
        )),
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

    let new_alias = crate::model::notification_alias::NewNotificationAlias {
        name: name.to_string(),
        description: req
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        recipients,
        is_enabled: req.get("is_enabled").and_then(|v| v.as_bool()),
    };

    match web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().unwrap();
            NotificationAlias::create(&new_alias, &mut conn)
        }
    })
    .await
    {
        Ok(Ok(id)) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Alias created",
                Some(format!("Alias ID: {}", id)),
            )),
        ),
        Ok(Err((_, msg))) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to create alias", Some(msg), 0),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to create alias",
                Some(e.to_string()),
                0,
            ),
        )),
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

    let update = crate::model::notification_alias::UpdateNotificationAlias {
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

    match web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().unwrap();
            NotificationAlias::update(alias_id, &update, &mut conn)
        }
    })
    .await
    {
        Ok(Ok(num)) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Alias updated",
                Some(format!("Rows affected: {}", num)),
            )),
        ),
        Ok(Err((_, msg))) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to update alias", Some(msg), 0),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to update alias",
                Some(e.to_string()),
                0,
            ),
        )),
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

    match web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().unwrap();
            NotificationAlias::delete(alias_id, &mut conn)
        }
    })
    .await
    {
        Ok(Ok(num)) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Alias deleted",
                Some(format!("Rows affected: {}", num)),
            )),
        ),
        Ok(Err((_, msg))) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(500, "Failed to delete alias", Some(msg), 0),
        )),
        Err(e) => Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(
                500,
                "Failed to delete alias",
                Some(e.to_string()),
                0,
            ),
        )),
    }
}
