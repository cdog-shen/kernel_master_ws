//! 查询过滤器与发送请求的清洗层辅助函数。
//!
//! 过滤白名单与 model 层 `get_with_filter` 支持的键及期望的值类型保持一致。
//! GET query 参数解析出的值总是字符串，因此清洗时会把白名单中 bool/int 型
//! 键的字符串值转换为原生类型；未知键与类型不匹配（含转换失败）的值会在
//! 过滤 map 下传前被剔除，这与 model 层静默忽略的语义等价，因此不会引入 400 错误。
//!
//! 发送请求的输入结构体取代了 handler 中逐字段的手工提取；其 `from_map`
//! 构造函数沿用了 handler 原先完全相同的默认值策略。

use serde_json::{Map, Value};

/// 白名单过滤键期望的 JSON 值类型。
#[derive(Clone, Copy)]
enum FilterValueType {
    Str,
    Bool,
    // 预留给未来的整型过滤键（当前白名单尚无 int 键）
    #[allow(dead_code)]
    Int,
}

impl FilterValueType {
    /// 将值按期望类型归一化：原生类型直接放行；
    /// 字符串值（GET query 参数解析出的值总是字符串）尝试按目标类型转换，
    /// 即 "true"/"false" → bool、数字字符串 → i64；
    /// 转换失败或类型不符返回 None，该键将被剔除（保持静默忽略语义）。
    fn coerce(self, value: Value) -> Option<Value> {
        match self {
            FilterValueType::Str => value.is_string().then_some(value),
            FilterValueType::Bool => match value {
                Value::Bool(_) => Some(value),
                Value::String(s) => match s.as_str() {
                    "true" => Some(Value::Bool(true)),
                    "false" => Some(Value::Bool(false)),
                    _ => None,
                },
                _ => None,
            },
            FilterValueType::Int => match value {
                Value::Number(ref n) if n.is_i64() => Some(value),
                Value::String(ref s) => s.parse::<i64>().ok().map(|n| Value::Number(n.into())),
                _ => None,
            },
        }
    }
}

/// 剔除白名单之外的键，并把字符串形式的 bool/int 值归一化为原生类型；
/// 类型不匹配或转换失败的值会被剔除。
fn clean_filter(
    filter: Map<String, Value>,
    whitelist: &[(&str, FilterValueType)],
) -> Map<String, Value> {
    filter
        .into_iter()
        .filter_map(|(key, value)| {
            whitelist
                .iter()
                .find(|(w_key, _)| *w_key == key)
                .and_then(|(_, w_type)| w_type.coerce(value))
                .map(|coerced| (key, coerced))
        })
        .collect()
}

/// `notification_records` 的过滤白名单，参见 `NotificationRecord::get_with_filter`。
const RECORD_FILTER_WHITELIST: &[(&str, FilterValueType)] = &[
    ("channel_type", FilterValueType::Str),
    ("status", FilterValueType::Str),
    ("recipient", FilterValueType::Str),
];

/// `notification_templates` 的过滤白名单，参见 `NotificationTemplate::get_with_filter`。
const TEMPLATE_FILTER_WHITELIST: &[(&str, FilterValueType)] = &[
    ("is_enabled", FilterValueType::Bool),
    ("name", FilterValueType::Str),
];

/// 清洗 `get_records` 接口的过滤条件。
pub fn clean_record_filter(filter: Map<String, Value>) -> Map<String, Value> {
    clean_filter(filter, RECORD_FILTER_WHITELIST)
}

/// 清洗 `get_templates` 接口的过滤条件。
pub fn clean_template_filter(filter: Map<String, Value>) -> Map<String, Value> {
    clean_filter(filter, TEMPLATE_FILTER_WHITELIST)
}

/// `POST /api/notify/template` 清洗后的输入。
pub struct SendTemplateRequestInput {
    pub template_name: Option<String>,
    pub variables: Map<String, Value>,
}

impl SendTemplateRequestInput {
    /// 从原始请求 map 中提取已知字段；`variables` 缺省为空对象，
    /// `template_name` 由 handler 负责校验。
    pub fn from_map(map: &Map<String, Value>) -> Self {
        SendTemplateRequestInput {
            template_name: map
                .get("template_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            variables: map
                .get("variables")
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_default(),
        }
    }
}
