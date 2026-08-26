use async_trait::async_trait;
use serde_json::{Value, json};
use share_lib::infrastructure::http_client;

use crate::model::channel_config::WebhookConfig;
use crate::services::channel::{Channel, DispatchError, NotificationRequest, run_http_call};

/// 通用 Webhook 推送渠道实现
pub struct WebhookChannel;

impl WebhookChannel {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Channel for WebhookChannel {
    fn channel_type(&self) -> &'static str {
        "webhook"
    }

    async fn dispatch(
        &self,
        config: &Value,
        recipient: &str,
        request: &NotificationRequest,
    ) -> Result<(), DispatchError> {
        let webhook_config: WebhookConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid Webhook config: {}", e)))?;

        let webhook_url = if !recipient.is_empty() {
            recipient.to_string()
        } else {
            webhook_config.webhook_url.clone()
        };

        // content_format=json 时，params_template 整体作为 payload
        let payload = if request.format == "json" {
            if request.params.is_null() || request.params.as_object().map_or(true, |m| m.is_empty())
            {
                return Err(DispatchError::Abort(
                    "content_format=json requires params_template to be set".to_string(),
                ));
            }
            request.params.clone()
        } else {
            // 默认简单 JSON 格式
            let mut body = json!({
                "title": request.title,
                "body": request.body,
                "priority": request.priority,
            });

            if let Some(url) = request.params.get("url").and_then(|v| v.as_str()) {
                if !url.is_empty() {
                    body["url"] = json!(url);
                }
            } else if let Some(url) = &request.url {
                body["url"] = json!(url);
            }

            if !request.tags.is_empty() {
                body["tags"] = json!(request.tags);
            }

            body
        };

        run_http_call("Webhook", move || {
            http_client::post_json(&webhook_url, &[], &[], &payload)
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
                .get("body")
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
        let webhook_config: WebhookConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid Webhook config: {}", e)))?;

        let webhook_url = if !recipient.is_empty() {
            recipient.to_string()
        } else {
            webhook_config.webhook_url.clone()
        };

        // Webhook 固定使用 JSON 格式，渲染后的 payload 原样发送
        let webhook_payload = payload.clone();

        run_http_call("Webhook", move || {
            http_client::post_json(&webhook_url, &[], &[], &webhook_payload)
        })
        .await
    }
}
