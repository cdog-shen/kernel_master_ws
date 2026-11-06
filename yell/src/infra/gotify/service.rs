use async_trait::async_trait;
use serde_json::{Value, json};
use share_lib::infrastructure::http_client;

use crate::model::channel_config::GotifyConfig;
use crate::service::channel::{
    Channel, DispatchError, collect_failures, run_http_call, string_elements,
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

    /// 校验配置与 string 元素（空串元素合法，表示回退到配置中的 app_token）
    async fn preflight(&self, config: &Value, recipients: &[Value]) -> Result<(), String> {
        serde_json::from_value::<GotifyConfig>(config.clone())
            .map_err(|e| format!("Invalid Gotify config: {}", e))?;
        string_elements(recipients, "Gotify")?;
        Ok(())
    }

    async fn send(
        &self,
        config: &Value,
        recipients: &[Value],
        payloads: &[Value],
    ) -> Result<(), DispatchError> {
        let gotify_config: GotifyConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid Gotify config: {}", e)))?;
        let tokens = string_elements(recipients, "Gotify").map_err(DispatchError::Abort)?;

        let payload = &payloads[0];
        let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let message = payload
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let priority = payload
            .get("priority")
            .and_then(|v| v.as_u64())
            .unwrap_or(5);

        let mut push_payload = json!({
            "title": title,
            "message": message,
            "priority": priority,
        });
        if let Some(url) = payload.get("url").and_then(|v| v.as_str())
            && !url.is_empty()
        {
            push_payload["extras"] = json!({
                "client::notification": {
                    "click": { "url": url }
                }
            });
        }

        let base_url = gotify_config.server_url.trim_end_matches('/').to_string();
        let push_url = format!("{}/message", base_url);
        let mut errors = Vec::new();
        for token in tokens {
            let app_token = if !token.is_empty() {
                token.to_string()
            } else {
                gotify_config.app_token.clone()
            };
            let url = push_url.clone();
            let query = [("token".to_string(), app_token)];
            let body = push_payload.clone();
            let result = run_http_call("Gotify", move || {
                http_client::post_json(&url, &[], &query, &body)
            })
            .await;
            if let Err(e) = result {
                errors.push(format!("app_token '{}': {}", token, e.into_msg()));
            }
        }
        collect_failures(errors)
    }
}
