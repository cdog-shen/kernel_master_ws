use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use ureq;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::webhook::*;

/// all_webhook api logic
pub fn all_webhook<'a>(
    filter: Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Vec<WebhookModel>>, MailManErr<'a, String>> {
    match WebhookModel::get_all_with_filter(&filter, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "All Webhook info", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    }
}

/// new_webhook api logic
pub fn new_webhook<'a>(
    webhook: WebhookInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match WebhookModel::new(&webhook, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Create webhook",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(
                400,
                "Service: Create webhook",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                500,
                "Service: Create webhook",
                Some(msg.1),
                1,
            )),
        },
    }
}

/// update_webhook api logic
pub fn update_webhook<'a>(
    webhook: WebhookInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match WebhookModel::update(&webhook, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Update webhook",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(
                400,
                "Service: Update webhook",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                500,
                "Service: Update webhook",
                Some(msg.1),
                1,
            )),
        },
    }
}

/// delete_webhook api logic
pub fn delete_webhook<'a>(
    id: i32,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match WebhookModel::delete(id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Delete webhook",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(
                400,
                "Service: Delete webhook",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                500,
                "Service: Delete webhook",
                Some(msg.1),
                1,
            )),
        },
    }
}

/// post_webhook api logic
pub fn post_webhook<'a>(
    hook_name: String,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, serde_json::Value>, MailManErr<'a, String>> {
    // 按 hook_name 查找 webhook 配置
    let mut filter = Map::new();
    filter.insert("hook_name".to_string(), Value::String(hook_name.clone()));
    filter.insert("is_enable".to_string(), Value::Bool(true));

    let webhook_config = match WebhookModel::get_all_with_filter(&filter, &mut pool.get().unwrap())
    {
        Ok(mut webhooks) => {
            if webhooks.is_empty() {
                return Err(MailManErr::new(
                    404,
                    "Service: Post webhook",
                    Some(format!("Webhook '{}' not found or disabled", hook_name)),
                    1,
                ));
            }
            webhooks.remove(0)
        }
        Err(msg) => {
            return Err(MailManErr::new(
                500,
                "Service: Post webhook",
                Some(msg.1),
                1,
            ));
        }
    };

    // 检查是否启用
    if !webhook_config.is_enable {
        return Err(MailManErr::new(
            400,
            "Service: Post webhook",
            Some("Webhook is disabled".to_string()),
            1,
        ));
    }

    let method = webhook_config.method_type.to_uppercase();
    let target_url = &webhook_config.target_url;

    // 根据 HTTP 方法分别处理（ureq 3.x GET/POST 返回不同类型）
    let result: Result<String, MailManErr<'_, String>> = match method.as_str() {
        "GET" | "DELETE" => {
            let mut req = match method.as_str() {
                "GET" => ureq::get(target_url),
                _ => ureq::delete(target_url),
            };
            if let Value::Object(header_map) = &webhook_config.header_json {
                for (key, value) in header_map {
                    if let Some(val_str) = value.as_str() {
                        req = req.header(key, val_str);
                    }
                }
            }
            if let Value::Object(query_map) = &webhook_config.query_json {
                for (key, value) in query_map {
                    if let Some(val_str) = value.as_str() {
                        req = req.query(key, val_str);
                    }
                }
            }
            req.call()
                .and_then(|mut r| r.body_mut().read_to_string())
                .map_err(|e| MailManErr::new(500, "Service: Post webhook", Some(e.to_string()), 1))
        }
        "POST" | "PUT" => {
            let mut req = match method.as_str() {
                "POST" => ureq::post(target_url),
                _ => ureq::put(target_url),
            };
            if let Value::Object(header_map) = &webhook_config.header_json {
                for (key, value) in header_map {
                    if let Some(val_str) = value.as_str() {
                        req = req.header(key, val_str);
                    }
                }
            }
            if let Value::Object(query_map) = &webhook_config.query_json {
                for (key, value) in query_map {
                    if let Some(val_str) = value.as_str() {
                        req = req.query(key, val_str);
                    }
                }
            }
            req.send_json(&webhook_config.body_json)
                .and_then(|mut r| r.body_mut().read_to_string())
                .map_err(|e| MailManErr::new(500, "Service: Post webhook", Some(e.to_string()), 1))
        }
        _ => {
            return Err(MailManErr::new(
                400,
                "Service: Post webhook",
                Some(format!("Unsupported HTTP method: {}", method)),
                1,
            ));
        }
    };

    match result {
        Ok(body) => {
            let json_body = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
            Ok(MailManOk::new(
                200,
                "Service: Post webhook",
                Some(json_body),
            ))
        }
        Err(err) => Err(err),
    }
}

/// get_webhook api logic (用于测试或获取webhook信息)
pub fn get_webhook<'a>(
    hook_name: String,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Vec<WebhookModel>>, MailManErr<'a, String>> {
    let mut filter = Map::new();
    filter.insert("hook_name".to_string(), Value::String(hook_name));

    match WebhookModel::get_all_with_filter(&filter, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Webhook info", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    }
}
