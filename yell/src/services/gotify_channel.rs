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

    fn build_message(&self, request: &NotificationRequest) -> Result<String, String> {
        Ok(request.body.clone())
    }

    /// Gotify 对非 text 格式降级为简介
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
        // 从数据库获取 Gotify 配置（按实例名）
        let gotify_config = web::block({
            let pool = pool.clone();
            let name = instance_name.to_string();
            move || {
                let mut conn = pool.get().map_err(|e| e.to_string())?;
                ChannelConfig::get_gotify_config_by_name(&name, &mut conn).map_err(|(_, msg)| msg)
            }
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e)?;

        let config = gotify_config.ok_or("Gotify config not found")?;

        // app_token: 优先使用 recipient，否则使用配置中的默认值
        let app_token = if !recipient.is_empty() {
            recipient.to_string()
        } else {
            config.app_token.clone()
        };

        // 创建通知记录
        let record_id = create_record("gotify", recipient, request, pool).await?;

        // 优先级映射: params 中的 priority 可覆盖
        let priority = request
            .params
            .get("priority")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or_else(|| match request.priority.as_str() {
                "low" => 2,
                "normal" => 5,
                "high" => 7,
                "urgent" => 10,
                _ => 5,
            });

        // 构建 payload
        let mut payload = json!({
            "title": request.title,
            "message": request.body,
            "priority": priority,
        });

        // 支持 extras 中的 click URL
        if let Some(url) = request.params.get("url").and_then(|v| v.as_str()) {
            if !url.is_empty() {
                payload["extras"] = json!({
                    "client::notification": {
                        "click": { "url": url }
                    }
                });
            }
        } else if let Some(url) = &request.url {
            payload["extras"] = json!({
                "client::notification": {
                    "click": { "url": url }
                }
            });
        }

        // 构建 form body
        let mut form_body = format!(
            "title={}&message={}&priority={}",
            payload["title"], payload["message"], priority
        );
        if let Some(extras) = payload.get("extras") {
            form_body.push_str(&format!("&extras={}", extras));
        }

        // 发送请求
        let push_url = format!(
            "{}/message?token={}",
            config.server_url.trim_end_matches('/'),
            app_token
        );
        let result = web::block(move || {
            ureq::post(&push_url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .send(form_body.as_str())
                .map_err(|e| format!("Gotify request error: {}", e))
        })
        .await;

        match result {
            Ok(Ok(_)) => {
                update_record(record_id, "sent", None, pool).await?;
                Ok(ChannelResult {
                    channel_type: "gotify".to_string(),
                    success: true,
                    record_id,
                    error_msg: None,
                })
            }
            Ok(Err(e)) => {
                update_record(record_id, "failed", Some(e.clone()), pool).await?;
                Ok(ChannelResult {
                    channel_type: "gotify".to_string(),
                    success: false,
                    record_id,
                    error_msg: Some(e),
                })
            }
            Err(e) => {
                let error_msg = format!("Gotify task error: {}", e);
                update_record(record_id, "failed", Some(error_msg.clone()), pool).await?;
                Ok(ChannelResult {
                    channel_type: "gotify".to_string(),
                    success: false,
                    record_id,
                    error_msg: Some(error_msg),
                })
            }
        }
    }
}
