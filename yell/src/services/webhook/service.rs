use async_trait::async_trait;
use serde_json::Value;
use share_lib::infrastructure::http_client;

use crate::model::channel_config::WebhookConfig;
use crate::services::channel::{
    Channel, DispatchError, collect_failures, run_http_call, string_elements,
};

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

    /// 校验配置与 string 元素（空串元素合法，表示回退到配置中的 webhook_url）
    async fn preflight(&self, config: &Value, recipients: &[Value]) -> Result<(), String> {
        serde_json::from_value::<WebhookConfig>(config.clone())
            .map_err(|e| format!("Invalid Webhook config: {}", e))?;
        string_elements(recipients, "Webhook")?;
        Ok(())
    }

    /// 渲染后的 payload 原样 POST 到每个 URL 元素；空串元素回退用配置中的 webhook_url
    async fn send(
        &self,
        config: &Value,
        recipients: &[Value],
        payloads: &[Value],
    ) -> Result<(), DispatchError> {
        let webhook_config: WebhookConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid Webhook config: {}", e)))?;
        let urls = string_elements(recipients, "Webhook").map_err(DispatchError::Abort)?;

        let mut errors = Vec::new();
        for url in urls {
            let webhook_url = if !url.is_empty() {
                url.to_string()
            } else {
                webhook_config.webhook_url.clone()
            };
            let webhook_payload = payloads[0].clone();
            let result = run_http_call("Webhook", move || {
                http_client::post_json(&webhook_url, &[], &[], &webhook_payload)
            })
            .await;
            if let Err(e) = result {
                errors.push(format!("url '{}': {}", url, e.into_msg()));
            }
        }
        collect_failures(errors)
    }
}
