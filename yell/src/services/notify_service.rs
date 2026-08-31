//! 发送侧编排层 —— recipients 解析（含 alias 查库）等发送前置编排
//!
//! 清洗层（api/notify.rs）只负责参数解析与校验，
//! 本模块负责组织 diesel 同步调用（web::block 包裹）并将结果映射为 MailMan 体系。

use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::Value;
use share_lib::data_structure::MailManErr;
use std::collections::HashMap;

use crate::model::channel_config::ChannelConfig;
use crate::model::notification_alias::NotificationAlias;
use crate::model::notification_template::NotificationTemplate;
use crate::services::channel::{ChannelConfigs, RecipientTarget};

/// 解析 recipients 字段，仅支持 String（alias）格式
pub async fn resolve_recipients<'a>(
    recipients_value: Option<&Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<Vec<RecipientTarget>, MailManErr<'a, String>> {
    match recipients_value {
        Some(Value::String(alias_name)) => {
            let alias_name = alias_name.clone();
            let result = web::block({
                let pool = pool.clone();
                let name = alias_name.clone();
                move || {
                    let mut conn = pool.get().map_err(|e| (0, e.to_string()))?;
                    NotificationAlias::get_by_name(&name, &mut conn)
                }
            })
            .await;

            match result {
                Ok(Ok(Some(alias))) => {
                    let arr = alias.recipients.as_array().ok_or_else(|| {
                        MailManErr::new(
                            500,
                            "Internal Error",
                            Some("Alias recipients is not an array".to_string()),
                            0,
                        )
                    })?;
                    parse_recipients_array(arr)
                }
                Ok(Ok(None)) => Err(MailManErr::new(
                    404,
                    "Not Found",
                    Some(format!("Alias '{}' not found or disabled", alias_name)),
                    1,
                )),
                Ok(Err((_, msg))) => Err(MailManErr::new(500, "Failed to get alias", Some(msg), 0)),
                Err(e) => Err(MailManErr::new(
                    500,
                    "Failed to get alias",
                    Some(e.to_string()),
                    0,
                )),
            }
        }
        Some(Value::Array(_)) => Err(MailManErr::new(
            400,
            "Bad Request",
            Some(
                "'recipients' only supports an alias name string; raw recipient arrays are no longer accepted"
                    .to_string(),
            ),
            1,
        )),
        _ => Err(MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'recipients' field (must be an alias name string)".to_string()),
            1,
        )),
    }
}

/// 解析 recipients JSON 数组为 Vec<RecipientTarget>
fn parse_recipients_array(
    arr: &[Value],
) -> Result<Vec<RecipientTarget>, MailManErr<'static, String>> {
    let mut recipients = Vec::new();
    for v in arr {
        let obj = v.as_object().ok_or_else(|| {
            MailManErr::new(
                400,
                "Bad Request",
                Some("Each recipient must be an object".to_string()),
                1,
            )
        })?;
        let channel_type = obj
            .get("channel_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing 'channel_type' in recipient".to_string()),
                    1,
                )
            })?;
        let instance = obj
            .get("instance")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing 'instance' in recipient".to_string()),
                    1,
                )
            })?;
        let recipient_arr = obj
            .get("recipient")
            .and_then(|v| v.as_array())
            .filter(|a| !a.is_empty())
            .ok_or_else(|| {
                MailManErr::new(
                    400,
                    "Bad Request",
                    Some(
                        "Field 'recipient' must be a non-empty array (plain string recipients are no longer supported)"
                            .to_string(),
                    ),
                    1,
                )
            })?;
        recipients.push(RecipientTarget {
            channel_type: channel_type.to_string(),
            instance: instance.to_string(),
            recipients: recipient_arr.clone(),
        });
    }
    Ok(recipients)
}

/// 校验 recipients 字段的结构（alias 写入侧使用）：
/// 顶层数组、每个元素为对象、channel_type/instance 为 string、recipient 为非空数组
pub fn validate_recipients_shape(value: &Value) -> Result<(), String> {
    let arr = value
        .as_array()
        .ok_or_else(|| "'recipients' must be an array".to_string())?;
    for v in arr {
        let obj = v
            .as_object()
            .ok_or_else(|| "Each recipient must be an object".to_string())?;
        for key in ["channel_type", "instance"] {
            if obj.get(key).and_then(|v| v.as_str()).is_none() {
                return Err(format!("Missing or non-string '{}' in recipient", key));
            }
        }
        match obj.get("recipient").and_then(|v| v.as_array()) {
            Some(a) if !a.is_empty() => {}
            _ => {
                return Err("Field 'recipient' must be a non-empty array".to_string());
            }
        }
    }
    Ok(())
}

/// 加载所有启用的渠道配置，返回 channel_type -> instance_name -> config_json 嵌套表
/// 只加载 is_enabled=true 的配置（渠道发送时按实例名取用，不再自行查库）
pub async fn load_channel_configs<'a>(
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<ChannelConfigs, MailManErr<'a, String>> {
    let result = web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().map_err(|e| e.to_string())?;
            ChannelConfig::get_all(&mut conn).map_err(|(_, msg)| msg)
        }
    })
    .await;

    let configs = match result {
        Ok(Ok(configs)) => configs,
        Ok(Err(msg)) => {
            return Err(MailManErr::new(
                500,
                "Failed to load channel configs",
                Some(msg),
                0,
            ));
        }
        Err(e) => {
            return Err(MailManErr::new(
                500,
                "Failed to load channel configs",
                Some(e.to_string()),
                0,
            ));
        }
    };

    let mut map: ChannelConfigs = HashMap::new();
    for config in configs {
        if config.get("is_enabled").and_then(|v| v.as_bool()) != Some(true) {
            continue;
        }
        if let (Some(channel_type), Some(instance), Some(config_json)) = (
            config.get("channel_type").and_then(|v| v.as_str()),
            config.get("name").and_then(|v| v.as_str()),
            config.get("config_json").cloned(),
        ) {
            map.entry(channel_type.to_string())
                .or_default()
                .insert(instance.to_string(), config_json);
        }
    }
    Ok(map)
}

/// 根据名称获取启用的模板；不存在时返回 404
pub async fn get_template_by_name<'a>(
    template_name: &str,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<NotificationTemplate, MailManErr<'a, String>> {
    let result = web::block({
        let pool = pool.clone();
        let name = template_name.to_string();
        move || {
            let mut conn = pool.get().map_err(|e| e.to_string())?;
            NotificationTemplate::get_by_name(&name, &mut conn).map_err(|(_, msg)| msg)
        }
    })
    .await;

    match result {
        Ok(Ok(Some(template))) => Ok(template),
        Ok(Ok(None)) => Err(MailManErr::new(
            404,
            "Template not found",
            Some(format!("Template '{}' does not exist", template_name)),
            1,
        )),
        Ok(Err(msg)) => Err(MailManErr::new(500, "Failed to get template", Some(msg), 0)),
        Err(e) => Err(MailManErr::new(
            500,
            "Failed to get template",
            Some(e.to_string()),
            0,
        )),
    }
}
