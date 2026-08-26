use async_trait::async_trait;
use serde_json::{Value, json};
use share_lib::infrastructure::http_client;

use crate::model::channel_config::GotifyConfig;
use crate::services::channel::{Channel, DispatchError, NotificationRequest, run_http_call};

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

    /// Gotify 对非 text 格式降级为简介
    fn prepare_request(&self, mut request: NotificationRequest) -> NotificationRequest {
        if request.format != "text" {
            request.body = "请查看详情".to_string();
            request.format = "text".to_string();
        }
        request
    }

    async fn dispatch(
        &self,
        config: &Value,
        recipient: &str,
        request: &NotificationRequest,
    ) -> Result<(), DispatchError> {
        let gotify_config: GotifyConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid Gotify config: {}", e)))?;

        let app_token = if !recipient.is_empty() {
            recipient.to_string()
        } else {
            gotify_config.app_token.clone()
        };

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
            gotify_config.server_url.trim_end_matches('/'),
            app_token
        );
        run_http_call("Gotify", move || {
            http_client::post_raw(
                &push_url,
                &[(
                    "Content-Type".to_string(),
                    "application/x-www-form-urlencoded".to_string(),
                )],
                &[],
                &form_body,
            )
        })
        .await
    }

    fn template_request(&self, payload: &Value, template_id: Option<i32>) -> NotificationRequest {
        NotificationRequest {
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
            template_id,
            params: Value::Null,
        }
    }

    async fn dispatch_template(
        &self,
        config: &Value,
        recipient: &str,
        payload: &Value,
        _template_id: Option<i32>,
    ) -> Result<(), DispatchError> {
        let gotify_config: GotifyConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid Gotify config: {}", e)))?;

        let app_token = if !recipient.is_empty() {
            recipient.to_string()
        } else {
            gotify_config.app_token.clone()
        };

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
            gotify_config.server_url.trim_end_matches('/'),
            app_token
        );
        run_http_call("Gotify", move || {
            http_client::post_raw(
                &push_url,
                &[(
                    "Content-Type".to_string(),
                    "application/x-www-form-urlencoded".to_string(),
                )],
                &[],
                &form_body,
            )
        })
        .await
    }
}
