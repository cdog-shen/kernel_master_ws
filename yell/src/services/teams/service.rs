use actix_web::web;
use async_trait::async_trait;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use serde_json::json;

use crate::model::channel_config::ChannelConfig;
use crate::services::channel::{
    create_record, update_record, Channel, ChannelResult, NotificationRequest,
};

/// Microsoft Teams 推送渠道实现（通过 Incoming Webhook）
pub struct TeamsChannel;

impl TeamsChannel {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Channel for TeamsChannel {
    fn channel_type(&self) -> &'static str {
        "teams"
    }

    fn build_message(&self, request: &NotificationRequest) -> Result<String, String> {
        Ok(request.body.clone())
    }

    async fn send(
        &self,
        recipient: &str,
        instance_name: &str,
        request: &NotificationRequest,
        pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    ) -> Result<ChannelResult, String> {
        let teams_config = web::block({
            let pool = pool.clone();
            let name = instance_name.to_string();
            move || {
                let mut conn = pool.get().map_err(|e| e.to_string())?;
                ChannelConfig::get_teams_config_by_name(&name, &mut conn).map_err(|(_, msg)| msg)
            }
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e)?;

        let config = teams_config.ok_or("Teams config not found")?;

        let webhook_url = if !recipient.is_empty() {
            recipient.to_string()
        } else {
            config.webhook_url.clone()
        };

        let record_id = create_record("teams", recipient, request, pool).await?;

        // content_format=json 时，params_template 整体作为 payload
        let payload = if request.format == "json" {
            if request.params.is_null() || request.params.as_object().map_or(true, |m| m.is_empty()) {
                return Err("content_format=json requires params_template to be set".to_string());
            }
            request.params.clone()
        } else {
            // 默认 MessageCard 格式
            let mut card = json!({
                "@type": "MessageCard",
                "@context": "http://schema.org/extensions",
                "summary": request.title,
                "themeColor": match request.priority.as_str() {
                    "urgent" => "FF0000",
                    "high" => "FF8C00",
                    "low" => "808080",
                    _ => "0076D7",
                },
                "title": request.title,
                "text": request.body,
            });

            if let Some(url) = request.params.get("url").and_then(|v| v.as_str()) {
                if !url.is_empty() {
                    card["potentialAction"] = json!([{
                        "@type": "OpenUri",
                        "name": "查看详情",
                        "targets": [{ "os": "default", "uri": url }]
                    }]);
                }
            } else if let Some(url) = &request.url {
                card["potentialAction"] = json!([{
                    "@type": "OpenUri",
                    "name": "查看详情",
                    "targets": [{ "os": "default", "uri": url }]
                }]);
            }
            card
        };

        let result = web::block(move || {
            ureq::post(&webhook_url)
                .send_json(&payload)
                .map_err(|e| format!("Teams request error: {}", e))
        })
        .await;

        match result {
            Ok(Ok(_)) => {
                update_record(record_id, "sent", None, pool).await?;
                Ok(ChannelResult {
                    channel_type: "teams".to_string(),
                    success: true,
                    record_id,
                    error_msg: None,
                })
            }
            Ok(Err(e)) => {
                update_record(record_id, "failed", Some(e.clone()), pool).await?;
                Ok(ChannelResult {
                    channel_type: "teams".to_string(),
                    success: false,
                    record_id,
                    error_msg: Some(e),
                })
            }
            Err(e) => {
                let error_msg = format!("Teams task error: {}", e);
                update_record(record_id, "failed", Some(error_msg.clone()), pool).await?;
                Ok(ChannelResult {
                    channel_type: "teams".to_string(),
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
        let teams_config = web::block({
            let pool = pool.clone();
            let name = instance_name.to_string();
            move || {
                let mut conn = pool.get().map_err(|e| e.to_string())?;
                ChannelConfig::get_teams_config_by_name(&name, &mut conn).map_err(|(_, msg)| msg)
            }
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e)?;

        let config = teams_config.ok_or("Teams config not found")?;

        let webhook_url = if !recipient.is_empty() {
            recipient.to_string()
        } else {
            config.webhook_url.clone()
        };

        let record_id = create_record("teams", recipient, &NotificationRequest {
            title: payload.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            body: payload.get("text").or_else(|| payload.get("body")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
            format: "text".to_string(),
            priority: "normal".to_string(),
            tags: vec![],
            url: None,
            mentions: vec![],
            template_id: _template_id,
            params: serde_json::Value::Null,
        }, pool).await?;

        // Teams 固定使用 MessageCard 格式，用 payload 中的字段构造
        let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let body = payload.get("text").or_else(|| payload.get("body")).and_then(|v| v.as_str()).unwrap_or("");
        let mut teams_payload = json!({
            "@type": "MessageCard",
            "@context": "http://schema.org/extensions",
            "summary": title,
            "themeColor": "0076D7",
            "title": title,
            "text": body,
        });
        if let Some(url) = payload.get("url").and_then(|v| v.as_str()) {
            if !url.is_empty() {
                teams_payload["potentialAction"] = json!([{
                    "@type": "OpenUri",
                    "name": "查看详情",
                    "targets": [{ "os": "default", "uri": url }]
                }]);
            }
        }

        let result = web::block(move || {
            ureq::post(&webhook_url)
                .send_json(&teams_payload)
                .map_err(|e| format!("Teams request error: {}", e))
        })
        .await;

        match result {
            Ok(Ok(_)) => {
                update_record(record_id, "sent", None, pool).await?;
                Ok(ChannelResult {
                    channel_type: "teams".to_string(),
                    success: true,
                    record_id,
                    error_msg: None,
                })
            }
            Ok(Err(e)) => {
                update_record(record_id, "failed", Some(e.clone()), pool).await?;
                Ok(ChannelResult {
                    channel_type: "teams".to_string(),
                    success: false,
                    record_id,
                    error_msg: Some(e),
                })
            }
            Err(e) => {
                let error_msg = format!("Teams task error: {}", e);
                update_record(record_id, "failed", Some(error_msg.clone()), pool).await?;
                Ok(ChannelResult {
                    channel_type: "teams".to_string(),
                    success: false,
                    record_id,
                    error_msg: Some(error_msg),
                })
            }
        }
    }
}
