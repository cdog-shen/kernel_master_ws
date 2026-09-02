use std::collections::HashMap;
use std::sync::Arc;

use actix_web::web;
use async_trait::async_trait;
use chrono::Local;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use share_lib::data_structure::MailManErr;

use crate::model::notification_record::{
    NewNotificationRecord, NotificationRecord, UpdateNotificationRecord,
};

/// 统一消息请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRequest {
    pub title: String,
    pub body: String,
    pub format: String,   // text / html / markdown
    pub priority: String, // low / normal / high / urgent
    pub tags: Vec<String>,
    pub url: Option<String>,
    pub mentions: Vec<String>,
    pub template_id: Option<i32>,
    pub params: Value, // 渠道扩展参数（Bark: level/sound/group/icon 等）
}

/// 渠道推送结果
#[derive(Debug, Clone, Serialize)]
pub struct ChannelResult {
    pub channel_type: String,
    pub success: bool,
    pub record_id: i32,
    pub error_msg: Option<String>,
}

/// 注入的渠道配置表：channel_type -> instance_name -> config_json
/// 由编排层（notify_service::load_channel_configs）统一加载后注入，
/// 渠道内不再自行查询 ChannelConfig
pub type ChannelConfigs = HashMap<String, HashMap<String, Value>>;

/// 单个接收目标：alias recipients 数组中一个元素解析后的结果
/// recipients 为原始 JSON 元素数组，元素 schema 由各渠道自行定义与校验
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipientTarget {
    pub channel_type: String,
    pub instance: String,
    pub recipients: Vec<Value>,
}

/// 渠道外呼失败类型
#[derive(Debug)]
pub enum DispatchError {
    /// 前置失败（参数/地址校验、配置解析失败等）：通知记录保持 pending，错误直接上抛
    Abort(String),
    /// 外呼已发起但失败：通知记录标记 failed，以 success=false 的 ChannelResult 返回
    Failed(String),
}

impl DispatchError {
    /// 提取错误文本（元素级失败聚合时使用）
    pub fn into_msg(self) -> String {
        match self {
            DispatchError::Abort(m) | DispatchError::Failed(m) => m,
        }
    }
}

/// 渠道 trait —— 所有通知渠道必须实现
/// 渠道只负责"校验/渲染/外呼"本身；配置获取与通知记录生命周期由本模块的
/// 统一骨架（deliver_template）管理
#[async_trait]
pub trait Channel: Send + Sync {
    /// 渠道类型标识
    fn channel_type(&self) -> &'static str;

    /// 外呼前置准备（默认无操作）
    /// 在创建通知记录之前调用；返回 Err 时中止发送，不创建记录。
    /// 渠道在此校验注入的配置、recipient 元素 schema、初始化惰性资源（如 SMTP transport）
    async fn preflight(&self, config: &Value, recipients: &[Value]) -> Result<(), String> {
        let _ = (config, recipients);
        Ok(())
    }

    /// 渲染本渠道 payload（默认实现：与 recipient 元素无关，渲染一次）
    /// template_json 为本渠道在模板中的 JSONB 列内容；
    /// 返回的 payloads[i] 与 recipients[i] 按下标一一对应（默认实现恒为 1 个）
    fn render(
        &self,
        recipients: &[Value],
        template_json: &Value,
        variables: &Map<String, Value>,
    ) -> Result<Vec<Value>, DispatchError> {
        let _ = recipients;
        Ok(vec![crate::service::template_render::render_channel(
            template_json,
            variables,
        )])
    }

    /// 执行外呼 —— 各渠道的核心发送方法
    /// config 为按 channel_type + instance_name 从注入配置中取出的 config_json；
    /// 元素间互不阻断，全部尝试完毕后有错才返回 Failed（错误信息汇总并标注失败元素）
    async fn send(
        &self,
        config: &Value,
        recipients: &[Value],
        payloads: &[Value],
    ) -> Result<(), DispatchError>;
}

/// 由渲染后的模板 payload 构造落库用的请求视图
/// 从 payload 提取 title/body/message，其余字段取默认值
fn record_view_from_payload(payload: &Value, template_id: Option<i32>) -> NotificationRequest {
    NotificationRequest {
        title: payload
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        body: payload
            .get("body")
            .or_else(|| payload.get("message"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        format: "text".to_string(),
        priority: "normal".to_string(),
        tags: vec![],
        url: payload
            .get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        mentions: vec![],
        template_id,
        params: payload.clone(),
    }
}

/// 统一发送流程骨架（模板发送）：
/// 取注入的配置 → preflight（校验配置与元素 schema）→ render →
/// 创建记录(pending) → send → update_record(sent/failed)
pub async fn deliver_template(
    channel: &Arc<dyn Channel>,
    target: &RecipientTarget,
    template_json: &Value,
    variables: &Map<String, Value>,
    template_id: Option<i32>,
    channel_configs: &ChannelConfigs,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<ChannelResult, String> {
    let config = resolve_config(channel_configs, channel.channel_type(), &target.instance)?;
    channel.preflight(config, &target.recipients).await?;
    let payloads = channel
        .render(&target.recipients, template_json, variables)
        .map_err(DispatchError::into_msg)?;

    let recipient_text = serde_json::to_string(&target.recipients)
        .map_err(|e| format!("Failed to serialize recipients: {}", e))?;
    let record_id = create_record(
        channel.channel_type(),
        &recipient_text,
        &record_view_from_payload(&payloads[0], template_id),
        pool,
    )
    .await?;

    let outcome = channel.send(config, &target.recipients, &payloads).await;
    finalize_delivery(outcome, channel.channel_type(), record_id, pool).await
}

/// 根据外呼结果更新记录状态并构造 ChannelResult
async fn finalize_delivery(
    outcome: Result<(), DispatchError>,
    channel_type: &str,
    record_id: i32,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<ChannelResult, String> {
    match outcome {
        Ok(()) => {
            update_record(record_id, "sent", None, pool).await?;
            Ok(ChannelResult {
                channel_type: channel_type.to_string(),
                success: true,
                record_id,
                error_msg: None,
            })
        }
        Err(DispatchError::Failed(e)) => {
            update_record(record_id, "failed", Some(e.clone()), pool).await?;
            Ok(ChannelResult {
                channel_type: channel_type.to_string(),
                success: false,
                record_id,
                error_msg: Some(e),
            })
        }
        // 前置失败：记录保持 pending，错误直接上抛
        Err(DispatchError::Abort(e)) => Err(e),
    }
}

/// 从注入的配置表中按 channel_type + instance_name 取配置
fn resolve_config<'c>(
    channel_configs: &'c ChannelConfigs,
    channel_type: &str,
    instance_name: &str,
) -> Result<&'c Value, String> {
    channel_configs
        .get(channel_type)
        .and_then(|instances| instances.get(instance_name))
        .ok_or_else(|| format!("{} config not found", channel_display_name(channel_type)))
}

/// 提取 string 类型的 recipient 元素；任一元素非 string 时返回错误信息
/// prefix 为渠道显示名（如 "Bark"）。空串元素原样保留，语义由各渠道解释
/// （bark/gotify/webhook/teams 中空串表示"回退到实例配置"）
pub fn string_elements<'r>(recipients: &'r [Value], prefix: &str) -> Result<Vec<&'r str>, String> {
    recipients
        .iter()
        .map(|v| {
            v.as_str()
                .ok_or_else(|| format!("{} recipient elements must be strings", prefix))
        })
        .collect()
}

/// 汇总元素级失败：无错误返回 Ok，否则 Failed（"; " 连接全部错误）
pub fn collect_failures(errors: Vec<String>) -> Result<(), DispatchError> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(DispatchError::Failed(errors.join("; ")))
    }
}

/// 渠道显示名（用于 "Xxx config not found" 错误信息，保持与原各渠道实现一致）
fn channel_display_name(channel_type: &str) -> &str {
    match channel_type {
        "smtp" => "SMTP",
        "bark" => "Bark",
        "gotify" => "Gotify",
        "teams" => "Teams",
        "teams_hook" => "TeamsHook",
        "webhook" => "Webhook",
        other => other,
    }
}

/// 在阻塞线程池执行同步 HTTP 外呼，并把结果映射为外呼成败
/// prefix 为渠道显示名（如 "Bark"），保持原 "Xxx request/task error" 错误格式
pub async fn run_http_call<F>(prefix: &str, call: F) -> Result<(), DispatchError>
where
    F: FnOnce() -> Result<String, MailManErr<'static, String>> + Send + 'static,
{
    match web::block(call).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(DispatchError::Failed(format!(
            "{} request error: {}",
            prefix,
            http_err_text(&e)
        ))),
        Err(e) => Err(DispatchError::Failed(format!(
            "{} task error: {}",
            prefix, e
        ))),
    }
}

/// 提取 MailManErr 的可读错误文本（供渠道包装 HTTP 外呼错误）
fn http_err_text(err: &MailManErr<'static, String>) -> String {
    err.msg.clone().unwrap_or_else(|| err.key.to_string())
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
            NotificationRecord::create(&new_record, &mut conn).map_err(|(_, msg)| msg)
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
            NotificationRecord::update_status(record_id, &update, &mut conn).map_err(|(_, msg)| msg)
        }
    })
    .await;

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(msg)) => Err(msg),
        Err(e) => Err(e.to_string()),
    }
}
