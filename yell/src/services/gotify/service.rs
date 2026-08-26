use actix_web::web;
use async_trait::async_trait;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::json;

use crate::model::channel_config::ChannelConfig;
use crate::services::channel::{
    Channel, ChannelResult, NotificationRequest, create_record, update_record,
};

/// Gotify 推送渠道实现
pub struct GotifyChannel;

impl GotifyChannel {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Channel for GotifyChannel {
    fn channel_type(&self) -> &'static str {
        "gotify"
    }

    fn build_message(&self, request: &NotificationRequest) -> Result<String, String> {
        Ok(request.body.clone())
    }

    /// Gotify 对非 text 格式降级为简介
    fn prepare_request(&self, mut request: NotificationRequest) -> NotificationRequest {
        if request.format != "text" {
            request.body = "请查看详情".to_string();
            request.format = "text".to_string();
        }
        request
    }

    async fn send(
        &self,
        recipient: &str,
        instance_name: &str,
        request: &NotificationRequest,
        pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    ) -> Result<ChannelResult, String> {
        let gotify_config = web::block({
            let pool = pool.clone();
            let name = instance_name.to_string();
            move || {
                let mut conn = pool.get().map_err(|e| e.to_string())?;
                ChannelConfig::get_gotify_config_by_name(&name, &mut conn).map_err(|(_, msg)| msg)
            }
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e)?;

        let config = gotify_config.ok_or("Gotify config not found")?;

        let app_token = if !recipient.is_empty() {
            recipient.to_string()
        } else {
            config.app_token.clone()
        };

        let record_id = create_record("gotify", recipient, request, pool).await?;

        let priority = request
            .params
            .get("priority")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or_else(|| match request.priority.as_str() {
                "low" => 2,
                "normal" => 5,
                "high" => 7,
                "urgent" => 10,
                _ => 5,
            });

        let mut payload = json!({
            "title": request.title,
            "message": request.body,
            "priority": priority,
        });

        if let Some(url) = request.params.get("url").and_then(|v| v.as_str()) {
            if !url.is_empty() {
                payload["extras"] = json!({
                    "client::notification": {
                        "click": { "url": url }
                    }
                });
            }
        } else if let Some(url) = &request.url {
            payload["extras"] = json!({
                "client::notification": {
                    "click": { "url": url }
                }
            });
        }

        let mut form_body = format!(
            "title={}&message={}&priority={}",
            payload["title"], payload["message"], priority
        );
        if let Some(extras) = payload.get("extras") {
            form_body.push_str(&format!("&extras={}", extras));
        }

        let push_url = format!(
            "{}/message?token={}",
            config.server_url.trim_end_matches('/'),
            app_token
        );
        let result = web::block(move || {
            ureq::post(&push_url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .send(form_body.as_str())
                .map_err(|e| format!("Gotify request error: {}", e))
        })
        .await;

        match result {
            Ok(Ok(_)) => {
                update_record(record_id, "sent", None, pool).await?;
                Ok(ChannelResult {
                    channel_type: "gotify".to_string(),
                    success: true,
                    record_id,
                    error_msg: None,
                })
            }
            Ok(Err(e)) => {
                update_record(record_id, "failed", Some(e.clone()), pool).await?;
                Ok(ChannelResult {
                    channel_type: "gotify".to_string(),
                    success: false,
                    record_id,
                    error_msg: Some(e),
                })
            }
            Err(e) => {
                let error_msg = format!("Gotify task error: {}", e);
                update_record(record_id, "failed", Some(error_msg.clone()), pool).await?;
                Ok(ChannelResult {
                    channel_type: "gotify".to_string(),
                    success: false,
                    record_id,
                    error_msg: Some(error_msg),
                })
            }
        }
    }

    async fn send_template(
        &self,
        recipient: &str,
        instance_name: &str,
        payload: &serde_json::Value,
        _template_id: Option<i32>,
        pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    ) -> Result<ChannelResult, String> {
        let gotify_config = web::block({
            let pool = pool.clone();
            let name = instance_name.to_string();
            move || {
                let mut conn = pool.get().map_err(|e| e.to_string())?;
                ChannelConfig::get_gotify_config_by_name(&name, &mut conn).map_err(|(_, msg)| msg)
            }
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e)?;

        let config = gotify_config.ok_or("Gotify config not found")?;

        let app_token = if !recipient.is_empty() {
            recipient.to_string()
        } else {
            config.app_token.clone()
        };

        let record_id = create_record(
            "gotify",
            recipient,
            &NotificationRequest {
                title: payload
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                body: payload
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                format: "text".to_string(),
                priority: "normal".to_string(),
                tags: vec![],
                url: None,
                mentions: vec![],
                template_id: _template_id,
                params: serde_json::Value::Null,
            },
            pool,
        )
        .await?;

        let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let message = payload
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let priority = payload
            .get("priority")
            .and_then(|v| v.as_u64())
            .unwrap_or(5);

        let mut form_body = format!("title={}&message={}&priority={}", title, message, priority);
        if let Some(url) = payload.get("url").and_then(|v| v.as_str()) {
            if !url.is_empty() {
                let extras = json!({
                    "client::notification": {
                        "click": { "url": url }
                    }
                });
                form_body.push_str(&format!("&extras={}", extras));
            }
        }

        let push_url = format!(
            "{}/message?token={}",
            config.server_url.trim_end_matches('/'),
            app_token
        );
        let result = web::block(move || {
            ureq::post(&push_url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .send(form_body.as_str())
                .map_err(|e| format!("Gotify request error: {}", e))
        })
        .await;

        match result {
            Ok(Ok(_)) => {
                update_record(record_id, "sent", None, pool).await?;
                Ok(ChannelResult {
                    channel_type: "gotify".to_string(),
                    success: true,
                    record_id,
                    error_msg: None,
                })
            }
            Ok(Err(e)) => {
                update_record(record_id, "failed", Some(e.clone()), pool).await?;
                Ok(ChannelResult {
                    channel_type: "gotify".to_string(),
                    success: false,
                    record_id,
                    error_msg: Some(e),
                })
            }
            Err(e) => {
                let error_msg = format!("Gotify task error: {}", e);
                update_record(record_id, "failed", Some(error_msg.clone()), pool).await?;
                Ok(ChannelResult {
                    channel_type: "gotify".to_string(),
                    success: false,
                    record_id,
                    error_msg: Some(error_msg),
                })
            }
        }
    }
}
