use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::notification_template::NotificationTemplate;
use crate::services::bark::service::BarkChannel;
use crate::services::channel::{Channel, ChannelResult, NotificationRequest};
use crate::services::gotify::service::GotifyChannel;
use crate::services::mail::service::SmtpChannel;
use crate::services::teams::service::TeamsChannel;
use crate::services::webhook::service::WebhookChannel;

/// 通知路由分发器
pub struct NotificationRouter {
    channels: HashMap<String, Arc<dyn Channel>>,
}

impl NotificationRouter {
    /// 创建新的路由器，注册所有可用渠道
    pub fn new() -> Self {
        let mut channels: HashMap<String, Arc<dyn Channel>> = HashMap::new();

        channels.insert("smtp".to_string(), Arc::new(SmtpChannel::new()));
        channels.insert("bark".to_string(), Arc::new(BarkChannel::new()));
        channels.insert("gotify".to_string(), Arc::new(GotifyChannel::new()));
        channels.insert("teams".to_string(), Arc::new(TeamsChannel::new()));
        channels.insert("webhook".to_string(), Arc::new(WebhookChannel::new()));

        // TODO: 注册更多渠道
        // channels.insert("wecom".to_string(), Arc::new(WeComChannel::new()));
        // channels.insert("lark".to_string(), Arc::new(LarkChannel::new()));
        // channels.insert("dingtalk".to_string(), Arc::new(DingtalkChannel::new()));

        Self { channels }
    }

    /// 发送消息到指定渠道列表
    ///
    /// recipients: Vec<(channel_type, recipient, instance)>
    /// 各 channel_type 自行解释 recipient 的含义（邮件地址/用户ID/群组ID/openID/token等）
    /// instance 为配置实例名，必填
    /// channel_configs: 由编排层注入的渠道配置（HashMap<channel_type, config_json>），
    /// 预留给渠道初始化使用；当前各渠道按 instance 自行查库加载配置
    pub async fn send_to_channels<'a>(
        &self,
        request: NotificationRequest,
        recipients: Vec<(String, String, String)>,
        channel_configs: HashMap<String, Value>,
        pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    ) -> Result<MailManOk<'a, Vec<ChannelResult>>, MailManErr<'a, String>> {
        if recipients.is_empty() {
            return Ok(MailManOk::new(
                200,
                "No recipients to send",
                Some(Vec::new()),
            ));
        }

        // 配置已注入但暂不消费（原 init_channel 为空实现，已移除）
        let _ = &channel_configs;

        let mut tasks = Vec::new();

        for (channel_type, recipient, instance) in recipients {
            if let Some(channel) = self.channels.get(&channel_type) {
                let channel = Arc::clone(channel);
                let request = request.clone();
                let pool = pool.clone();

                let task = tokio::spawn(async move {
                    let request = channel.prepare_request(request);
                    channel.send(&recipient, &instance, &request, &pool).await
                });

                tasks.push(task);
            }
        }

        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => {
                    results.push(ChannelResult {
                        channel_type: "unknown".to_string(),
                        success: false,
                        record_id: -1,
                        error_msg: Some(e),
                    });
                }
                Err(e) => {
                    results.push(ChannelResult {
                        channel_type: "unknown".to_string(),
                        success: false,
                        record_id: -1,
                        error_msg: Some(format!("Task join error: {}", e)),
                    });
                }
            }
        }

        Ok(MailManOk::new(200, "Notification processed", Some(results)))
    }

    /// 使用模板发送 — 渲染各渠道 JSON，只向模板中有对应字段的渠道发送
    ///
    /// template: 由编排层按名称查询后注入的模板对象
    pub async fn send_with_template<'a>(
        &self,
        template: NotificationTemplate,
        variables: serde_json::Map<String, Value>,
        recipients: Vec<(String, String, String)>,
        pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    ) -> Result<MailManOk<'a, Vec<ChannelResult>>, MailManErr<'a, String>> {
        // 渲染各渠道 JSON: HashMap<channel_type, rendered_payload>
        let rendered = template.render(&variables);
        let template_id = template.id;

        let mut tasks = Vec::new();

        for (channel_type, recipient, instance) in recipients {
            // 只向模板中配置了对应字段的渠道发送
            if let Some(payload) = rendered.get(&channel_type) {
                if let Some(channel) = self.channels.get(&channel_type) {
                    let channel = Arc::clone(channel);
                    let payload = payload.clone();
                    let pool = pool.clone();

                    let task = tokio::spawn(async move {
                        channel
                            .send_template(
                                &recipient,
                                &instance,
                                &payload,
                                Some(template_id),
                                &pool,
                            )
                            .await
                    });

                    tasks.push(task);
                }
            }
        }

        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => {
                    results.push(ChannelResult {
                        channel_type: "unknown".to_string(),
                        success: false,
                        record_id: -1,
                        error_msg: Some(e),
                    });
                }
                Err(e) => {
                    results.push(ChannelResult {
                        channel_type: "unknown".to_string(),
                        success: false,
                        record_id: -1,
                        error_msg: Some(format!("Task join error: {}", e)),
                    });
                }
            }
        }

        Ok(MailManOk::new(
            200,
            "Template notification processed",
            Some(results),
        ))
    }
}

impl Default for NotificationRouter {
    fn default() -> Self {
        Self::new()
    }
}
