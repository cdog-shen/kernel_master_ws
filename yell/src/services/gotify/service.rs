use async_trait::async_trait;
use serde_json::{Value, json};
use share_lib::infrastructure::http_client;

use crate::model::channel_config::GotifyConfig;
use crate::services::channel::{Channel, DispatchError, run_http_call};

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
