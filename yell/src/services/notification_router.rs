use actix_web::web;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::{
    channel_config::ChannelConfig,
    notification_template::NotificationTemplate,
};
use crate::services::bark_channel::BarkChannel;
use crate::services::channel::{Channel, ChannelResult, NotificationRequest};
use crate::services::smtp_channel::SmtpChannel;

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
    pub async fn send_to_channels<'a>(
        &self,
        request: NotificationRequest,
        recipients: Vec<(String, String, String)>,
        pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    ) -> Result<MailManOk<'a, Vec<ChannelResult>>, MailManErr<'a, String>> {
        if recipients.is_empty() {
            return Ok(MailManOk::new(
                200,
                "No recipients to send",
                Some(Vec::new()),
            ));
        }

        let channel_configs = match Self::load_channel_configs(pool).await {
            Ok(configs) => configs,
            Err(e) => {
                return Err(MailManErr::new(
                    500,
                    "Failed to load channel configs",
                    Some(e),
                    0,
                ));
            }
        };

        let mut tasks = Vec::new();

        for (channel_type, recipient, instance) in recipients {
            if let Some(channel) = self.channels.get(&channel_type) {
                let channel = Arc::clone(channel);
                let request = request.clone();
                let pool = pool.clone();
                let config = channel_configs.get(&channel_type).cloned();

                if let Some(config) = config {
                    if let Err(e) = Self::init_channel(&channel, &config).await {
                        return Err(MailManErr::new(
                            500,
                            "Failed to init channel",
                            Some(format!("{}: {}", channel_type, e)),
                            0,
                        ));
                    }
                }

                let task = tokio::spawn(async move {
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

        Ok(MailManOk::new(
            200,
            "Notification processed",
            Some(results),
        ))
    }

    /// 使用模板发送
    pub async fn send_with_template<'a>(
        &self,
        template_name: &str,
        variables: serde_json::Map<String, Value>,
        recipients: Vec<(String, String, String)>,
        pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    ) -> Result<MailManOk<'a, Vec<ChannelResult>>, MailManErr<'a, String>> {
        let template = web::block({
            let pool = pool.clone();
            let name = template_name.to_string();
            move || {
                let mut conn = pool.get().map_err(|e| e.to_string())?;
                NotificationTemplate::get_by_name(&name, &mut conn)
                    .map_err(|(_, msg)| msg)
            }
        })
        .await;

        let template = match template {
            Ok(Ok(Some(t))) => t,
            Ok(Ok(None)) => {
                return Err(MailManErr::new(
                    404,
                    "Template not found",
                    Some(format!("Template '{}' does not exist", template_name)),
                    1,
                ));
            }
            Ok(Err(msg)) => {
                return Err(MailManErr::new(
                    500,
                    "Failed to get template",
                    Some(msg),
                    0,
                ));
            }
            Err(e) => {
                return Err(MailManErr::new(
                    500,
                    "Failed to get template",
                    Some(e.to_string()),
                    0,
                ));
            }
        };

        let (subject, content) = template.render(&variables);

        let request = NotificationRequest {
            title: subject,
            body: content,
            format: template.content_format.unwrap_or_else(|| "text".to_string()),
            priority: "normal".to_string(),
            tags: Vec::new(),
            url: None,
            mentions: Vec::new(),
            template_id: Some(template.id),
        };

        self.send_to_channels(request, recipients, pool).await
    }

    /// 加载所有渠道配置
    async fn load_channel_configs(
        pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    ) -> Result<HashMap<String, Value>, String> {
        let result = web::block({
            let pool = pool.clone();
            move || {
                let mut conn = pool.get().map_err(|e| e.to_string())?;
                ChannelConfig::get_all(&mut conn).map_err(|(_, msg)| msg)
            }
        })
        .await;

        match result {
            Ok(Ok(configs)) => {
                let mut map = HashMap::new();
                for config in configs {
                    if let Ok(channel_type) = serde_json::from_value::<String>(
                        config.get("channel_type").cloned().unwrap_or(Value::Null),
                    ) {
                        map.insert(channel_type, config);
                    }
                }
                Ok(map)
            }
            Ok(Err(msg)) => Err(msg),
            Err(e) => Err(e.to_string()),
        }
    }

    /// 初始化渠道
    async fn init_channel(
        _channel: &Arc<dyn Channel>,
        _config: &Value,
    ) -> Result<(), String> {
        // TODO: 根据配置初始化渠道客户端
        // 例如：SMTP 可以在这里创建连接池
        Ok(())
    }
}

impl Default for NotificationRouter {
    fn default() -> Self {
        Self::new()
    }
}
