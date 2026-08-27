use async_trait::async_trait;
use serde_json::{Value, json};
use share_lib::infrastructure::http_client;

use crate::model::channel_config::BarkConfig;
use crate::services::channel::{Channel, DispatchError, run_http_call};

/// Bark 推送渠道实现
pub struct BarkChannel;

impl BarkChannel {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Channel for BarkChannel {
    fn channel_type(&self) -> &'static str {
        "bark"
    }

    async fn dispatch_template(
        &self,
        config: &Value,
        recipient: &str,
        payload: &Value,
        _template_id: Option<i32>,
    ) -> Result<(), DispatchError> {
        let bark_config: BarkConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid Bark config: {}", e)))?;

        let device_key = if !recipient.is_empty() {
            Some(recipient.to_string())
        } else {
            bark_config.device_key
        };

        let mut push_payload = payload.clone();
        if let Some(ref key) = device_key {
            push_payload["device_key"] = json!(key);
        }

        let push_url = format!("{}/push", bark_config.server_url.trim_end_matches('/'));
        run_http_call("Bark", move || {
            http_client::post_json(&push_url, &[], &[], &push_payload)
        })
        .await
    }
}
