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
use crate::services::channel::{
    Channel, ChannelConfigs, ChannelResult, RecipientTarget, deliver_template,
};
use crate::services::gotify::service::GotifyChannel;
use crate::services::mail::service::SmtpChannel;
use crate::services::teams::service::TeamsChannel;
use crate::services::teams_hook::service::TeamsHookChannel;
use crate::services::template_render;
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
        channels.insert("teams_hook".to_string(), Arc::new(TeamsHookChannel::new()));
        channels.insert("webhook".to_string(), Arc::new(WebhookChannel::new()));

        Self { channels }
    }

    /// 使用模板发送 — 只向模板中有对应字段的渠道发送
    ///
    /// template: 由编排层按名称查询后注入的模板对象
    /// channel_configs: 由编排层注入的渠道配置（channel_type -> instance_name -> config_json）
    /// 渲染已下沉到各渠道（Channel::render），本函数只负责路由与任务编排
    pub async fn send_with_template<'a>(
        &self,
        template: NotificationTemplate,
        variables: serde_json::Map<String, Value>,
        recipients: Vec<RecipientTarget>,
        channel_configs: ChannelConfigs,
        pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    ) -> Result<MailManOk<'a, Vec<ChannelResult>>, MailManErr<'a, String>> {
        let template_id = template.id;

        let mut tasks = Vec::new();

        for target in recipients {
            // 只向模板中配置了对应字段的渠道发送
            if let Some(template_json) =
                template_render::get_channel_json(&template, &target.channel_type)
                && let Some(channel) = self.channels.get(&target.channel_type)
            {
                let channel = Arc::clone(channel);
                let template_json = template_json.clone();
                let variables = variables.clone();
                let configs = channel_configs.clone();
                let pool = pool.clone();

                let task = tokio::spawn(async move {
                    deliver_template(
                        &channel,
                        &target,
                        &template_json,
                        &variables,
                        Some(template_id),
                        &configs,
                        &pool,
                    )
                    .await
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
