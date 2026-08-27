use async_trait::async_trait;
use serde_json::Value;
use share_lib::infrastructure::http_client;

use crate::model::channel_config::WebhookConfig;
use crate::services::channel::{Channel, DispatchError, run_http_call};

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

    async fn dispatch_template(
        &self,
        config: &Value,
        recipient: &str,
        payload: &Value,
        _template_id: Option<i32>,
    ) -> Result<(), DispatchError> {
        dispatch_webhook("Webhook", config, recipient, payload).await
    }
}

/// Webhook 核心发送实现（webhook 与 teams_hook 渠道共用）：
/// recipient 非空时优先作为目标 URL，否则取配置中的 webhook_url；
/// 渲染后的 payload 原样 POST 到目标 URL。
/// prefix 为渠道显示名，用于 "Invalid Xxx config" / "Xxx request error" 错误信息
pub(crate) async fn dispatch_webhook(
    prefix: &str,
    config: &Value,
    recipient: &str,
    payload: &Value,
) -> Result<(), DispatchError> {
    let webhook_config: WebhookConfig = serde_json::from_value(config.clone())
        .map_err(|e| DispatchError::Abort(format!("Invalid {} config: {}", prefix, e)))?;

    let webhook_url = if !recipient.is_empty() {
        recipient.to_string()
    } else {
        webhook_config.webhook_url.clone()
    };

    // Webhook 固定使用 JSON 格式，渲染后的 payload 原样发送
    let webhook_payload = payload.clone();

    run_http_call(prefix, move || {
        http_client::post_json(&webhook_url, &[], &[], &webhook_payload)
    })
    .await
}
