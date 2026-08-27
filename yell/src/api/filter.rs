//! 查询过滤器与发送请求的清洗层辅助函数。
//!
//! 过滤白名单与 model 层 `get_with_filter` 支持的键及期望的值类型保持一致；
//! 未知键与类型不匹配的值会在过滤 map 下传前被剔除，这与 model 层静默忽略
//! 的语义等价，因此不会引入 400 错误。
//!
//! 发送请求的输入结构体取代了 handler 中逐字段的手工提取；其 `from_map`
//! 构造函数沿用了 handler 原先完全相同的默认值策略。

use serde_json::{Map, Value};

/// 白名单过滤键期望的 JSON 值类型。
#[derive(Clone, Copy)]
enum FilterValueType {
    Str,
    Bool,
}

impl FilterValueType {
    fn matches(self, value: &Value) -> bool {
        match self {
            FilterValueType::Str => value.is_string(),
            FilterValueType::Bool => value.is_boolean(),
        }
    }
}

/// 剔除白名单之外的键以及类型不匹配的值。
fn clean_filter(
    filter: Map<String, Value>,
    whitelist: &[(&str, FilterValueType)],
) -> Map<String, Value> {
    filter
        .into_iter()
        .filter(|(key, value)| {
            whitelist
                .iter()
                .any(|(w_key, w_type)| *w_key == key && w_type.matches(value))
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
