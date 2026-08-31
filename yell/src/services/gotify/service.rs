use async_trait::async_trait;
use serde_json::{Value, json};
use share_lib::infrastructure::http_client;

use crate::model::channel_config::GotifyConfig;
use crate::services::channel::{
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

        let base_url = gotify_config.server_url.trim_end_matches('/').to_string();
        let mut errors = Vec::new();
        for token in tokens {
            let app_token = if !token.is_empty() {
                token.to_string()
            } else {
                gotify_config.app_token.clone()
            };
            let push_url = format!("{}/message?token={}", base_url, app_token);
            let body = form_body.clone();
            let result = run_http_call("Gotify", move || {
                http_client::post_raw(
                    &push_url,
                    &[(
                        "Content-Type".to_string(),
                        "application/x-www-form-urlencoded".to_string(),
                    )],
                    &[],
                    &body,
                )
            })
            .await;
            if let Err(e) = result {
                errors.push(format!("app_token '{}': {}", token, e.into_msg()));
            }
        }
        collect_failures(errors)
    }
}
