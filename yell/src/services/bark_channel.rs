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

    async fn send(
        &self,
        recipient: &str,
        request: &NotificationRequest,
        pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    ) -> Result<ChannelResult, String> {
        // 从数据库获取 Bark 配置
        let bark_config = web::block({
            let pool = pool.clone();
            move || {
                let mut conn = pool.get().map_err(|e| e.to_string())?;
                ChannelConfig::get_bark_config(&mut conn).map_err(|(_, msg)| msg)
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

        if let Some(url) = &request.url {
            payload["url"] = json!(url);
        }
        if !request.tags.is_empty() {
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
