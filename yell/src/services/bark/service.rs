use async_trait::async_trait;
use serde_json::{Value, json};
use share_lib::infrastructure::http_client;

use crate::model::channel_config::BarkConfig;
use crate::services::channel::{
    Channel, DispatchError, collect_failures, run_http_call, string_elements,
};

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

    /// 校验配置与 string 元素（空串元素合法，表示回退到配置中的 device_key）
    async fn preflight(&self, config: &Value, recipients: &[Value]) -> Result<(), String> {
        serde_json::from_value::<BarkConfig>(config.clone())
            .map_err(|e| format!("Invalid Bark config: {}", e))?;
        string_elements(recipients, "Bark")?;
        Ok(())
    }

    async fn send(
        &self,
        config: &Value,
        recipients: &[Value],
        payloads: &[Value],
    ) -> Result<(), DispatchError> {
        let bark_config: BarkConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid Bark config: {}", e)))?;
        let keys = string_elements(recipients, "Bark").map_err(DispatchError::Abort)?;

        let push_url = format!("{}/push", bark_config.server_url.trim_end_matches('/'));
        let mut errors = Vec::new();
        for key in keys {
            let device_key = if !key.is_empty() {
                Some(key.to_string())
            } else {
                bark_config.device_key.clone()
            };

            let mut push_payload = payloads[0].clone();
            if let Some(ref k) = device_key {
                push_payload["device_key"] = json!(k);
            }

            let url = push_url.clone();
            let result =
                run_http_call("Bark", move || http_client::post_json(&url, &[], &[], &push_payload))
                    .await;
            if let Err(e) = result {
                errors.push(format!("device_key '{}': {}", key, e.into_msg()));
            }
        }
        collect_failures(errors)
    }
}
