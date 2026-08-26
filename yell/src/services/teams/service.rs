use async_trait::async_trait;
use serde_json::{Value, json};
use share_lib::infrastructure::http_client;

use crate::model::channel_config::TeamsConfig;
use crate::services::channel::{Channel, DispatchError, NotificationRequest, run_http_call};

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

    async fn dispatch(
        &self,
        config: &Value,
        recipient: &str,
        request: &NotificationRequest,
    ) -> Result<(), DispatchError> {
        let teams_config: TeamsConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid Teams config: {}", e)))?;

        let webhook_url = if !recipient.is_empty() {
            recipient.to_string()
        } else {
            teams_config.webhook_url.clone()
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

        run_http_call("Teams", move || {
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
                .get("text")
                .or_else(|| payload.get("body"))
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
        let teams_config: TeamsConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid Teams config: {}", e)))?;

        let webhook_url = if !recipient.is_empty() {
            recipient.to_string()
        } else {
            teams_config.webhook_url.clone()
        };

        // Teams 固定使用 MessageCard 格式
        // 已知字段：title, text/body, url
        // 其余字段自动放入 facts 数组
        let known_keys: &[&str] = &["title", "text", "body", "url"];
        let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let body = payload
            .get("text")
            .or_else(|| payload.get("body"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mut teams_payload = json!({
            "@type": "MessageCard",
            "@context": "http://schema.org/extensions",
            "summary": title,
            "themeColor": "0076D7",
            "title": title,
            "text": body,
        });

        // 收集额外字段作为 facts
        if let Some(obj) = payload.as_object() {
            let facts: Vec<Value> = obj
                .iter()
                .filter(|(k, _)| !known_keys.contains(&k.as_str()))
                .filter_map(|(k, v)| {
                    let val = v.as_str().map(|s| s.to_string()).or_else(|| {
                        if v.is_number() || v.is_boolean() {
                            Some(v.to_string())
                        } else {
                            None
                        }
                    })?;
                    Some(json!({"name": k, "value": val}))
                })
                .collect();
            if !facts.is_empty() {
                teams_payload["sections"] = json!([{ "facts": facts }]);
            }
        }

        if let Some(url) = payload.get("url").and_then(|v| v.as_str()) {
            if !url.is_empty() {
                teams_payload["potentialAction"] = json!([{
                    "@type": "OpenUri",
                    "name": "查看详情",
                    "targets": [{ "os": "default", "uri": url }]
                }]);
            }
        }

        run_http_call("Teams", move || {
            http_client::post_json(&webhook_url, &[], &[], &teams_payload)
        })
        .await
    }
}
