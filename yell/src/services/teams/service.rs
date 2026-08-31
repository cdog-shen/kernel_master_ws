use async_trait::async_trait;
use serde_json::{Value, json};
use share_lib::infrastructure::http_client;

use crate::model::channel_config::TeamsConfig;
use crate::services::channel::{
    Channel, DispatchError, collect_failures, run_http_call, string_elements,
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

    /// 校验配置与 string 元素（空串元素合法，表示回退到配置中的 webhook_url）
    async fn preflight(&self, config: &Value, recipients: &[Value]) -> Result<(), String> {
        serde_json::from_value::<TeamsConfig>(config.clone())
            .map_err(|e| format!("Invalid Teams config: {}", e))?;
        string_elements(recipients, "Teams")?;
        Ok(())
    }

    async fn send(
        &self,
        config: &Value,
        recipients: &[Value],
        payloads: &[Value],
    ) -> Result<(), DispatchError> {
        let teams_config: TeamsConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid Teams config: {}", e)))?;
        let urls = string_elements(recipients, "Teams").map_err(DispatchError::Abort)?;

        // Teams 固定使用 MessageCard 格式
        // 已知字段：title, text/body, url
        // 其余字段自动放入 facts 数组
        let payload = &payloads[0];
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

        let mut errors = Vec::new();
        for url in urls {
            let webhook_url = if !url.is_empty() {
                url.to_string()
            } else {
                teams_config.webhook_url.clone()
            };
            let body = teams_payload.clone();
            let result = run_http_call("Teams", move || {
                http_client::post_json(&webhook_url, &[], &[], &body)
            })
            .await;
            if let Err(e) = result {
                errors.push(format!("url '{}': {}", url, e.into_msg()));
            }
        }
        collect_failures(errors)
    }
}
