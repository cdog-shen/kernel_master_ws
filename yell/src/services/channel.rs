use actix_web::web;
use async_trait::async_trait;
use chrono::Local;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::notification_record::{
    NewNotificationRecord, NotificationRecord, UpdateNotificationRecord,
};

/// 统一消息请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRequest {
    pub title: String,
    pub body: String,
    pub format: String,      // text / html / markdown
    pub priority: String,    // low / normal / high / urgent
    pub tags: Vec<String>,
    pub url: Option<String>,
    pub mentions: Vec<String>,
    pub template_id: Option<i32>,
    pub params: Value,       // 渠道扩展参数（Bark: level/sound/group/icon 等）
}

/// 渠道推送结果
#[derive(Debug, Clone, Serialize)]
pub struct ChannelResult {
    pub channel_type: String,
    pub success: bool,
    pub record_id: i32,
    pub error_msg: Option<String>,
}

/// 渠道配置上下文
#[derive(Debug, Clone)]
pub struct ChannelContext {
    pub channel_type: String,
    pub config: Value,
}

/// 渠道 trait —— 所有通知渠道必须实现
#[async_trait]
pub trait Channel: Send + Sync {
    /// 渠道类型标识
    fn channel_type(&self) -> &'static str;

    /// 构建渠道特定的消息体
    fn build_message(&self, request: &NotificationRequest) -> Result<String, String>;

    /// 发送前适配请求内容（默认不修改）
    /// 简单推送渠道（Bark/Gotify 等）可覆写此方法，对 HTML 等不适配格式做降级处理
    fn prepare_request(&self, request: NotificationRequest) -> NotificationRequest {
        request
    }

    /// 直接发送消息
    async fn send(
        &self,
        recipient: &str,
        instance_name: &str,
        request: &NotificationRequest,
        pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    ) -> Result<ChannelResult, String>;

    /// 通过模板发送 — payload 是渲染后的渠道专属 JSON
    /// 默认实现：从 payload 提取 title/body/message 构造 NotificationRequest，回退到 send()
    /// 各渠道可覆写此方法以直接使用 payload JSON 构造请求
    async fn send_template(
        &self,
        recipient: &str,
        instance_name: &str,
        payload: &Value,
        template_id: Option<i32>,
        pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    ) -> Result<ChannelResult, String> {
        let request = NotificationRequest {
            title: payload.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            body: payload.get("body")
                .or_else(|| payload.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            format: "text".to_string(),
            priority: "normal".to_string(),
            tags: vec![],
            url: payload.get("url").and_then(|v| v.as_str()).map(|s| s.to_string()),
            mentions: vec![],
            template_id,
            params: payload.clone(),
        };
        self.send(recipient, instance_name, &request, pool).await
    }
}

/// 创建通知记录
pub async fn create_record(
    channel_type: &str,
    recipient: &str,
    request: &NotificationRequest,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<i32, String> {
    let new_record = NewNotificationRecord {
        channel_type: channel_type.to_string(),
        recipient: recipient.to_string(),
        subject: Some(request.title.clone()),
        content: request.body.clone(),
        content_format: Some(request.format.clone()),
        template_id: request.template_id,
        status: Some("pending".to_string()),
        error_msg: None,
        sent_at: None,
    };

    let result = web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().map_err(|e| e.to_string())?;
            NotificationRecord::create(&new_record, &mut conn)
                .map_err(|(_, msg)| msg)
        }
    })
    .await;

    match result {
        Ok(Ok(id)) => Ok(id),
        Ok(Err(msg)) => Err(msg),
        Err(e) => Err(e.to_string()),
    }
}

/// 更新通知记录状态
pub async fn update_record(
    record_id: i32,
    status: &str,
    error_msg: Option<String>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<(), String> {
    let update = UpdateNotificationRecord {
        status: Some(status.to_string()),
        error_msg,
        sent_at: if status == "sent" {
            Some(Local::now().naive_local())
        } else {
            None
        },
        updated_at: Some(Local::now().naive_local()),
    };

    let result = web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().map_err(|e| e.to_string())?;
            NotificationRecord::update_status(record_id, &update, &mut conn)
                .map_err(|(_, msg)| msg)
        }
    })
    .await;

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(msg)) => Err(msg),
        Err(e) => Err(e.to_string()),
    }
}
