//! 模板渲染 —— 递归替换模板各渠道 JSONB 字段中的 {{var}} 占位符
//!
//! 业务逻辑自 model/notification_template.rs 迁出，model 只保留结构体与 diesel 查询。

use serde_json::{Map, Value};
use std::collections::HashMap;

use crate::model::notification_template::NotificationTemplate;

/// 渠道 JSONB 字段名列表
const CHANNEL_COLUMNS: &[&str] = &["smtp", "bark", "gotify", "teams_hook", "webhook"];

/// 渲染模板，返回 HashMap<channel_type, rendered_payload_json>
/// 只渲染模板中已配置的渠道字段
pub fn render_template(
    template: &NotificationTemplate,
    variables: &Map<String, Value>,
) -> HashMap<String, Value> {
    let mut result = HashMap::new();
    for col in CHANNEL_COLUMNS {
        if let Some(json_val) = get_channel_json(template, col) {
            let rendered = render_value(json_val, variables);
            result.insert(col.to_string(), rendered);
        }
    }
    result
}

/// 获取指定渠道的 JSONB 字段
fn get_channel_json<'t>(template: &'t NotificationTemplate, col: &str) -> Option<&'t Value> {
    match col {
        "smtp" => template.smtp.as_ref(),
        "bark" => template.bark.as_ref(),
        "gotify" => template.gotify.as_ref(),
        "teams_hook" => template.teams_hook.as_ref(),
        "webhook" => template.webhook.as_ref(),
        _ => None,
    }
}

fn replace_vars(template: &str, variables: &Map<String, Value>) -> String {
    let mut result = template.to_string();
    for (key, value) in variables.iter() {
        let placeholder = format!("{{{{{}}}}}", key);
        let value_str = value.as_str().unwrap_or("").to_string();
        result = result.replace(&placeholder, &value_str);
    }
    result
}

/// 递归渲染 JSON Value 中的 {{var}} 占位符
fn render_value(value: &Value, variables: &Map<String, Value>) -> Value {
    match value {
        Value::String(s) => Value::String(replace_vars(s, variables)),
        Value::Object(map) => {
            let mut rendered = serde_json::Map::new();
            for (k, v) in map {
                rendered.insert(k.clone(), render_value(v, variables));
            }
            Value::Object(rendered)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(|v| render_value(v, variables)).collect()),
        other => other.clone(),
    }
}
