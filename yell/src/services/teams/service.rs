use async_trait::async_trait;
use serde_json::{Value, json};
use share_lib::infrastructure::http_client;

use crate::model::channel_config::TeamsConfig;
use crate::services::channel::{Channel, DispatchError, run_http_call};

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
