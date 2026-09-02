use async_trait::async_trait;
use serde_json::{Map, Value};
use share_lib::infrastructure::http_client;

use crate::model::channel_config::WebhookConfig;
use crate::service::channel::{Channel, DispatchError, collect_failures, run_http_call};
use crate::service::template_render;

/// Teams Hook 推送渠道实现
/// 是通用 webhook 的一层封装：recipient 元素为对象，
/// 模板用原始 variables 渲染一次（不感知接收人信息），
/// 每个元素的字段（user/group_id/team_id/channel_id）再 merge 到渲染结果的顶层，
/// 逐元素 POST 到配置中的 webhook_url
pub struct TeamsHookChannel;

/// recipient 元素对象允许的 key
const ALLOWED_KEYS: &[&str] = &["user", "group_id", "team_id", "channel_id"];

impl TeamsHookChannel {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Channel for TeamsHookChannel {
    fn channel_type(&self) -> &'static str {
        "teams_hook"
    }

    /// 校验配置与元素 schema：对象、key 合法且值为 string、至少出现一个允许的 key
    async fn preflight(&self, config: &Value, recipients: &[Value]) -> Result<(), String> {
        serde_json::from_value::<WebhookConfig>(config.clone())
            .map_err(|e| format!("Invalid TeamsHook config: {}", e))?;
        for v in recipients {
            let obj = v
                .as_object()
                .ok_or_else(|| "TeamsHook recipient elements must be objects".to_string())?;
            if obj.is_empty() {
                return Err(
                    "TeamsHook recipient element must contain at least one of user/group_id/team_id/channel_id"
                        .to_string(),
                );
            }
            for (k, val) in obj {
                if !ALLOWED_KEYS.contains(&k.as_str()) {
                    return Err(format!("Unknown TeamsHook recipient key '{}'", k));
                }
                if !val.is_string() {
                    return Err(format!("TeamsHook recipient key '{}' must be a string", k));
                }
            }
        }
        Ok(())
    }

    /// 先用原始 variables 渲染出基础 payload（模板不感知接收人信息），
    /// 再把每个元素的字段 merge 到渲染结果的顶层；payloads[i] 与 recipients[i] 一一对应
    fn render(
        &self,
        recipients: &[Value],
        template_json: &Value,
        variables: &Map<String, Value>,
    ) -> Result<Vec<Value>, DispatchError> {
        let base = template_render::render_channel(template_json, variables);
        let mut payloads = Vec::with_capacity(recipients.len());
        for v in recipients {
            let obj = v.as_object().ok_or_else(|| {
                DispatchError::Abort("TeamsHook recipient elements must be objects".to_string())
            })?;
            let mut payload = base.clone();
            let map = payload.as_object_mut().ok_or_else(|| {
                DispatchError::Abort("TeamsHook rendered payload must be a JSON object".to_string())
            })?;
            for (k, val) in obj {
                map.insert(k.clone(), val.clone());
            }
            payloads.push(payload);
        }
        Ok(payloads)
    }

    /// 逐元素 POST 到配置中的 webhook_url
    async fn send(
        &self,
        config: &Value,
        _recipients: &[Value],
        payloads: &[Value],
    ) -> Result<(), DispatchError> {
        let hook_config: WebhookConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid TeamsHook config: {}", e)))?;

        let mut errors = Vec::new();
        for (i, payload) in payloads.iter().enumerate() {
            let url = hook_config.webhook_url.clone();
            let body = payload.clone();
            let result = run_http_call("TeamsHook", move || {
                http_client::post_json(&url, &[], &[], &body)
            })
            .await;
            if let Err(e) = result {
                errors.push(format!("element {}: {}", i, e.into_msg()));
            }
        }
        collect_failures(errors)
    }
}
