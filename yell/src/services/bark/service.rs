use async_trait::async_trait;
use serde_json::{Value, json};
use share_lib::infrastructure::http_client;

use crate::model::channel_config::BarkConfig;
use crate::services::channel::{Channel, DispatchError, NotificationRequest, run_http_call};

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

    /// Bark 不支持 HTML，对非 text 格式降级为简介
    fn prepare_request(&self, mut request: NotificationRequest) -> NotificationRequest {
        if request.format != "text" {
            request.body = "请查看详情".to_string();
            request.format = "text".to_string();
        }
        request
    }

    async fn dispatch(
        &self,
        config: &Value,
        recipient: &str,
        request: &NotificationRequest,
    ) -> Result<(), DispatchError> {
        let bark_config: BarkConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid Bark config: {}", e)))?;

        // device_key: 优先使用 recipient，否则使用配置中的默认值，都为空则发送给全体
        let device_key = if !recipient.is_empty() {
            Some(recipient.to_string())
        } else {
            bark_config.device_key
        };

        // 构建 Bark 请求体
        let mut payload = json!({
            "title": request.title,
            "body": request.body,
        });

        // 有 device_key 时指定设备，否则发送给连接到此 server 的全体用户
        if let Some(ref key) = device_key {
            payload["device_key"] = json!(key);
        }

        // 从 params 中读取 Bark 扩展参数
        if let Some(level) = request.params.get("level").and_then(|v| v.as_str()) {
            payload["level"] = json!(level);
        }
        if let Some(sound) = request.params.get("sound").and_then(|v| v.as_str()) {
            payload["sound"] = json!(sound);
        }
        if let Some(icon) = request.params.get("icon").and_then(|v| v.as_str()) {
            payload["icon"] = json!(icon);
        }
        if let Some(copy) = request.params.get("copy").and_then(|v| v.as_str()) {
            payload["copy"] = json!(copy);
        }
        if let Some(is_archive) = request.params.get("isArchive").and_then(|v| v.as_i64()) {
            payload["isArchive"] = json!(is_archive);
        }
        if let Some(automatically_copy) = request
            .params
            .get("automaticallyCopy")
            .and_then(|v| v.as_i64())
        {
            payload["automaticallyCopy"] = json!(automatically_copy);
        }

        // url: params 中的优先，否则用 request.url
        if let Some(url) = request.params.get("url").and_then(|v| v.as_str()) {
            if !url.is_empty() {
                payload["url"] = json!(url);
            }
        } else if let Some(url) = &request.url {
            payload["url"] = json!(url);
        }

        // group: params 中的优先，否则用 tags
        if let Some(group) = request.params.get("group").and_then(|v| v.as_str()) {
            if !group.is_empty() {
                payload["group"] = json!(group);
            }
        } else if !request.tags.is_empty() {
            payload["group"] = json!(request.tags.join(","));
        }

        // 发送请求
        let push_url = format!("{}/push", bark_config.server_url.trim_end_matches('/'));
        run_http_call("Bark", move || {
            http_client::post_json(&push_url, &[], &[], &payload)
        })
        .await
    }

    fn template_request(&self, payload: &Value, template_id: Option<i32>) -> NotificationRequest {
        NotificationRequest {
            title: payload
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            body: payload
                .get("body")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            format: "text".to_string(),
            priority: "normal".to_string(),
            tags: vec![],
            url: None,
            mentions: vec![],
            template_id,
            params: Value::Null,
        }
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
