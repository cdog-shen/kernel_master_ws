use actix_web::web;
use async_trait::async_trait;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message,
};
use serde_json::Value;
use std::str::FromStr;
use tokio::sync::Mutex;

use crate::model::channel_config::{ChannelConfig};
use crate::services::channel::{
    create_record, update_record, Channel, ChannelResult, NotificationRequest,
};

/// SMTP 渠道实现
pub struct SmtpChannel {
    transport: Mutex<Option<AsyncSmtpTransport<lettre::Tokio1Executor>>>,
}

impl SmtpChannel {
    pub fn new() -> Self {
        Self {
            transport: Mutex::new(None),
        }
    }

    /// 初始化 SMTP 客户端（懒加载，第一次发送时初始化）
    async fn init_transport(
        &self,
        instance_name: &str,
        pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    ) -> Result<(), String> {
        let mut transport_lock = self.transport.lock().await;

        if transport_lock.is_some() {
            return Ok(());
        }

        // 从数据库获取 SMTP 配置（按实例名）
        let smtp_config = web::block({
            let pool = pool.clone();
            let name = instance_name.to_string();
            move || {
                let mut conn = pool.get().map_err(|e| e.to_string())?;
                ChannelConfig::get_smtp_config_by_name(&name, &mut conn)
                    .map_err(|(_, msg)| msg)
            }
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e)?;

        let config = smtp_config.ok_or("SMTP config not found")?;

        let creds = Credentials::new(config.username, config.password);

        let transport = if config.use_tls {
            AsyncSmtpTransport::<lettre::Tokio1Executor>::starttls_relay(&config.host)
                .map_err(|e| format!("SMTP relay error: {}", e))?
                .port(config.port)
                .credentials(creds)
                .build()
        } else {
            AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(&config.host)
                .port(config.port)
                .credentials(creds)
                .build()
        };

        *transport_lock = Some(transport);
        Ok(())
    }
}

#[async_trait]
impl Channel for SmtpChannel {
    fn channel_type(&self) -> &'static str {
        "smtp"
    }

    fn build_message(&self, request: &NotificationRequest) -> Result<String, String> {
        // SMTP 直接返回 body 内容
        Ok(request.body.clone())
    }

    async fn send(
        &self,
        recipient: &str,
        instance_name: &str,
        request: &NotificationRequest,
        pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    ) -> Result<ChannelResult, String> {
        // 初始化传输器（懒加载）
        self.init_transport(instance_name, pool).await?;

        // 创建通知记录
        let record_id = create_record("smtp", recipient, request, pool).await?;

        // 获取传输器
        let transport = {
            let lock = self.transport.lock().await;
            lock
                .as_ref()
                .ok_or("SMTP transport not initialized")?
                .clone()
        };

        // 获取 SMTP 配置中的 from 地址（按实例名）
        let from_addr = web::block({
            let pool = pool.clone();
            let name = instance_name.to_string();
            move || {
                let mut conn = pool.get().map_err(|e| e.to_string())?;
                let config = ChannelConfig::get_smtp_config_by_name(&name, &mut conn)
                    .map_err(|(_, msg)| msg)?;
                config.ok_or("SMTP config not found".to_string())
                    .map(|c| c.from)
            }
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e)?;

        // 解析收件人列表（支持逗号或分号分隔）
        let addresses: Vec<&str> = recipient
            .split(|c| c == ',' || c == ';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if addresses.is_empty() {
            return Err("No valid recipient addresses provided".to_string());
        }

        let from_mailbox = Mailbox::from_str(&from_addr)
            .map_err(|e| format!("Invalid from address: {}", e))?;

        let content_type = if request.format == "html" {
            ContentType::TEXT_HTML
        } else {
            ContentType::TEXT_PLAIN
        };

        let mut builder = Message::builder().from(from_mailbox);
        for addr in &addresses {
            let mailbox = Mailbox::from_str(addr)
                .map_err(|e| format!("Invalid recipient address '{}': {}", addr, e))?;
            builder = builder.to(mailbox);
        }

        let email = builder
            .subject(request.title.clone())
            .header(content_type)
            .body(request.body.clone())
            .map_err(|e| format!("Failed to build email: {}", e))?;

        // 发送邮件
        match transport.send(email).await {
            Ok(_) => {
                update_record(record_id, "sent", None, pool).await?;
                Ok(ChannelResult {
                    channel_type: "smtp".to_string(),
                    success: true,
                    record_id,
                    error_msg: None,
                })
            }
            Err(e) => {
                let error_msg = format!("SMTP send error: {}", e);
                update_record(record_id, "failed", Some(error_msg.clone()), pool).await?;
                Ok(ChannelResult {
                    channel_type: "smtp".to_string(),
                    success: false,
                    record_id,
                    error_msg: Some(error_msg),
                })
            }
        }
    }
}
