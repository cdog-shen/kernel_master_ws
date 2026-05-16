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

    /// 发送消息
    async fn send(
        &self,
        recipient: &str,
        instance_name: &str,
        request: &NotificationRequest,
        pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    ) -> Result<ChannelResult, String>;
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
