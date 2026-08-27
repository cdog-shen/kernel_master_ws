use async_trait::async_trait;
use serde_json::Value;

use crate::services::channel::{Channel, DispatchError};
use crate::services::webhook::service::dispatch_webhook;

/// Teams Incoming Webhook 推送渠道实现
/// 行为与通用 Webhook 渠道一致：recipient 即 webhook URL，
/// 渲染后的 payload 原样 POST（复用 webhook 模块的核心发送实现）
pub struct TeamsHookChannel;

impl TeamsHookChannel {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Channel for TeamsHookChannel {
    fn channel_type(&self) -> &'static str {
        "teams_hook"
    }

    async fn dispatch_template(
        &self,
        config: &Value,
        recipient: &str,
        payload: &Value,
        _template_id: Option<i32>,
    ) -> Result<(), DispatchError> {
        dispatch_webhook("TeamsHook", config, recipient, payload).await
    }
}
