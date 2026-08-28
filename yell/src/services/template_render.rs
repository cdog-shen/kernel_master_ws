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

/// 生成模板的模拟渲染示例，返回 HashMap<channel_type, example_payload_json>
/// 只计算模板中已配置的渠道字段，输出形态与 render_template 一致
///
/// 变量类型推断规则：
/// - 渠道字段为 JSONB 列，占位符 {{var}} 只能出现在 JSON 字符串值内
///   （裸占位符不是合法 JSON，无法入库）；
/// - 渲染机制（replace_vars）仅以字符串形式注入变量值，
///   因此所有占位符统一推断为 String 类型，mock 值为 "TEST"；
/// - 非占位符的字面量（数字、布尔、普通字符串）原样保留；
/// - 任何无法推断的场景一律回退为 "TEST"。
pub fn render_example(template: &NotificationTemplate) -> HashMap<String, Value> {
    let mut mock_vars = Map::new();
    for var in collect_placeholders(template) {
        mock_vars.insert(var, Value::String("TEST".to_string()));
    }
    render_template(template, &mock_vars)
}

/// 收集模板所有渠道 JSONB 字段中出现的 {{var}} 占位符变量名（去重）
fn collect_placeholders(template: &NotificationTemplate) -> Vec<String> {
    let mut vars = Vec::new();
    for col in CHANNEL_COLUMNS {
        if let Some(json_val) = get_channel_json(template, col) {
            collect_placeholders_in_value(json_val, &mut vars);
        }
    }
    vars
}

/// 递归收集 JSON Value 字符串中的占位符变量名
fn collect_placeholders_in_value(value: &Value, vars: &mut Vec<String>) {
    match value {
        Value::String(s) => {
            for name in extract_placeholders(s) {
                if !vars.contains(&name) {
                    vars.push(name);
                }
            }
        }
        Value::Object(map) => {
            for v in map.values() {
                collect_placeholders_in_value(v, vars);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                collect_placeholders_in_value(v, vars);
            }
        }
        _ => {}
    }
}

/// 提取字符串中的 {{var}} 占位符变量名（忽略空名占位符）
fn extract_placeholders(s: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = s;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        match after.find("}}") {
            Some(end) => {
                let name = &after[..end];
                if !name.is_empty() {
                    names.push(name.to_string());
                }
                rest = &after[end + 2..];
            }
            None => break,
        }
    }
    names
}
