use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use tokio::sync::Mutex;

use crate::model::channel_config::SmtpConfig;
use crate::services::channel::{Channel, DispatchError};

/// SMTP 渠道实现
pub struct SmtpChannel {
    /// 按实例配置缓存 transport，避免多实例时错误复用同一连接
    transports: Mutex<HashMap<String, AsyncSmtpTransport<lettre::Tokio1Executor>>>,
}

impl SmtpChannel {
    pub fn new() -> Self {
        Self {
            transports: Mutex::new(HashMap::new()),
        }
    }

    /// 由连接相关的配置字段生成缓存 key（配置不变则 key 稳定）
    fn transport_key(smtp_config: &SmtpConfig) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            smtp_config.host,
            smtp_config.port,
            smtp_config.username,
            smtp_config.password,
            smtp_config.use_tls
        )
    }

    /// 初始化 SMTP 客户端（懒加载，该实例第一次发送时初始化）
    async fn init_transport(&self, config: &Value) -> Result<(), String> {
        let smtp_config: SmtpConfig = serde_json::from_value(config.clone())
            .map_err(|e| format!("Invalid SMTP config: {}", e))?;

        let key = Self::transport_key(&smtp_config);

        let mut transport_lock = self.transports.lock().await;

        if transport_lock.contains_key(&key) {
            return Ok(());
        }

        let creds = Credentials::new(smtp_config.username, smtp_config.password);

        let transport = if smtp_config.use_tls {
            AsyncSmtpTransport::<lettre::Tokio1Executor>::starttls_relay(&smtp_config.host)
                .map_err(|e| format!("SMTP relay error: {}", e))?
                .port(smtp_config.port)
                .credentials(creds)
                .build()
        } else {
            AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(&smtp_config.host)
                .port(smtp_config.port)
                .credentials(creds)
                .build()
        };

        transport_lock.insert(key, transport);
        Ok(())
    }

    /// 取指定实例已初始化的 transport（由 preflight 保证已初始化）
    async fn transport(
        &self,
        key: &str,
    ) -> Result<AsyncSmtpTransport<lettre::Tokio1Executor>, DispatchError> {
        let lock = self.transports.lock().await;
        lock.get(key)
            .cloned()
            .ok_or_else(|| DispatchError::Abort("SMTP transport not initialized".to_string()))
    }

    /// 解析收件人地址列表（逗号/分号分隔）
    fn parse_addresses(recipient: &str) -> Result<Vec<&str>, DispatchError> {
        let addresses: Vec<&str> = recipient
            .split(|c| c == ',' || c == ';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if addresses.is_empty() {
            return Err(DispatchError::Abort(
                "No valid recipient addresses provided".to_string(),
            ));
        }
        Ok(addresses)
    }

    /// 构建邮件
    fn build_email(
        from: &str,
        addresses: &[&str],
        subject: &str,
        content_type: ContentType,
        body: &str,
    ) -> Result<Message, DispatchError> {
        let from_mailbox = Mailbox::from_str(from)
            .map_err(|e| DispatchError::Abort(format!("Invalid from address: {}", e)))?;

        let mut builder = Message::builder().from(from_mailbox);
        for addr in addresses {
            let mailbox = Mailbox::from_str(addr).map_err(|e| {
                DispatchError::Abort(format!("Invalid recipient address '{}': {}", addr, e))
            })?;
            builder = builder.to(mailbox);
        }

        builder
            .subject(subject.to_string())
            .header(content_type)
            .body(body.to_string())
            .map_err(|e| DispatchError::Abort(format!("Failed to build email: {}", e)))
    }
}

#[async_trait]
impl Channel for SmtpChannel {
    fn channel_type(&self) -> &'static str {
        "smtp"
    }

    /// 发送前确保 SMTP transport 已初始化
    async fn preflight(&self, config: &Value) -> Result<(), String> {
        self.init_transport(config).await
    }

    async fn dispatch_template(
        &self,
        config: &Value,
        recipient: &str,
        payload: &Value,
        _template_id: Option<i32>,
    ) -> Result<(), DispatchError> {
        let smtp_config: SmtpConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid SMTP config: {}", e)))?;

        let transport = self.transport(&Self::transport_key(&smtp_config)).await?;
        let addresses = Self::parse_addresses(recipient)?;

        let subject = payload
            .get("subject")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let body = payload.get("body").and_then(|v| v.as_str()).unwrap_or("");
        let content_type = if payload.get("content_type").and_then(|v| v.as_str()) == Some("html") {
            ContentType::TEXT_HTML
        } else {
            ContentType::TEXT_PLAIN
        };

        let email = Self::build_email(&smtp_config.from, &addresses, subject, content_type, body)?;

        match transport.send(email).await {
            Ok(_) => Ok(()),
            Err(e) => Err(DispatchError::Failed(format!("SMTP send error: {}", e))),
        }
    }
}
