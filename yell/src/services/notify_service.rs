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

use crate::model::notification_alias::NotificationAlias;

/// 解析 recipients 字段，支持 String（alias）和 Array 两种格式
pub async fn resolve_recipients<'a>(
    recipients_value: Option<&Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<Vec<(String, String, String)>, MailManErr<'a, String>> {
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
        Some(Value::Array(arr)) => parse_recipients_array(arr),
        _ => Err(MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'recipients' field (must be an array or alias name string)".to_string()),
            1,
        )),
    }
}

/// 解析 recipients JSON 数组为 Vec<(channel_type, recipient, instance)>
fn parse_recipients_array(
    arr: &[Value],
) -> Result<Vec<(String, String, String)>, MailManErr<'static, String>> {
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
        let recipient = obj
            .get("recipient")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing 'recipient' in recipient".to_string()),
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
        recipients.push((
            channel_type.to_string(),
            recipient.to_string(),
            instance.to_string(),
        ));
    }
    Ok(recipients)
}
