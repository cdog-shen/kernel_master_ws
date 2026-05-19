use actix_web::web;
use async_trait::async_trait;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use serde_json::json;

use crate::model::channel_config::ChannelConfig;
use crate::services::channel::{
    create_record, update_record, Channel, ChannelResult, NotificationRequest,
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

    fn build_message(&self, request: &NotificationRequest) -> Result<String, String> {
        Ok(request.body.clone())
    }

    /// Bark 不支持 HTML，对非 text 格式降级为简介
    fn prepare_request(&self, mut request: NotificationRequest) -> NotificationRequest {
        if request.format != "text" {
            request.body = "请查看详情".to_string();
            request.format = "text".to_string();
        }
        request
    }

    async fn send(
        &self,
        recipient: &str,
        instance_name: &str,
        request: &NotificationRequest,
        pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    ) -> Result<ChannelResult, String> {
        // 从数据库获取 Bark 配置（按实例名）
        let bark_config = web::block({
            let pool = pool.clone();
            let name = instance_name.to_string();
            move || {
                let mut conn = pool.get().map_err(|e| e.to_string())?;
                ChannelConfig::get_bark_config_by_name(&name, &mut conn).map_err(|(_, msg)| msg)
            }
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e)?;

        let config = bark_config.ok_or("Bark config not found")?;

        // device_key: 优先使用 recipient，否则使用配置中的默认值
        let device_key = if !recipient.is_empty() {
            recipient.to_string()
        } else {
            config
                .device_key
                .ok_or("No device_key provided (neither in recipient nor config)")?
        };

        // 创建通知记录
        let record_id = create_record("bark", recipient, request, pool).await?;

        // 构建 Bark 请求体
        let mut payload = json!({
            "device_key": device_key,
            "title": request.title,
            "body": request.body,
        });

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
        if let Some(automatically_copy) = request.params.get("automaticallyCopy").and_then(|v| v.as_i64()) {
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
        let push_url = format!("{}/push", config.server_url.trim_end_matches('/'));
        let result = web::block(move || {
            ureq::post(&push_url)
                .send_json(&payload)
                .map_err(|e| format!("Bark request error: {}", e))
        })
        .await;

        match result {
            Ok(Ok(_)) => {
                update_record(record_id, "sent", None, pool).await?;
                Ok(ChannelResult {
                    channel_type: "bark".to_string(),
                    success: true,
                    record_id,
                    error_msg: None,
                })
            }
            Ok(Err(e)) => {
                update_record(record_id, "failed", Some(e.clone()), pool).await?;
                Ok(ChannelResult {
                    channel_type: "bark".to_string(),
                    success: false,
                    record_id,
                    error_msg: Some(e),
                })
            }
            Err(e) => {
                let error_msg = format!("Bark task error: {}", e);
                update_record(record_id, "failed", Some(error_msg.clone()), pool).await?;
                Ok(ChannelResult {
                    channel_type: "bark".to_string(),
                    success: false,
                    record_id,
                    error_msg: Some(error_msg),
                })
            }
        }
    }
}
