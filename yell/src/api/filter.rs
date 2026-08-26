//! Cleaning-layer helpers for query filters and send requests.
//!
//! Filter whitelists mirror the keys supported by the model's
//! `get_with_filter` together with the expected value type; unknown keys and
//! wrongly-typed values are stripped before the filter map is passed down,
//! which is semantically equivalent to the model's silent ignoring, so no 400
//! error is introduced.
//!
//! The send-request input structs replace manual per-field extraction in the
//! handlers; their `from_map` constructors apply the exact same default
//! policy the handlers used before.

use serde_json::{Map, Value};

use crate::services::channel::NotificationRequest;

/// Expected JSON value type for a whitelisted filter key.
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

/// Strip keys outside the whitelist and values of the wrong type.
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

/// Whitelist for `notification_records`, see `NotificationRecord::get_with_filter`.
const RECORD_FILTER_WHITELIST: &[(&str, FilterValueType)] = &[
    ("channel_type", FilterValueType::Str),
    ("status", FilterValueType::Str),
    ("recipient", FilterValueType::Str),
];

/// Whitelist for `notification_templates`, see `NotificationTemplate::get_with_filter`.
const TEMPLATE_FILTER_WHITELIST: &[(&str, FilterValueType)] = &[
    ("is_enabled", FilterValueType::Bool),
    ("name", FilterValueType::Str),
];

/// Clean the filter for the `get_records` endpoint.
pub fn clean_record_filter(filter: Map<String, Value>) -> Map<String, Value> {
    clean_filter(filter, RECORD_FILTER_WHITELIST)
}

/// Clean the filter for the `get_templates` endpoint.
pub fn clean_template_filter(filter: Map<String, Value>) -> Map<String, Value> {
    clean_filter(filter, TEMPLATE_FILTER_WHITELIST)
}

/// Extract a string array field, dropping non-string items; missing or
/// wrongly-typed fields yield an empty list.
fn string_list(map: &Map<String, Value>, key: &str) -> Vec<String> {
    map.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// Cleaned input of `POST /api/notify/send`.
pub struct SendRequestInput {
    pub title: String,
    pub body: String,
    pub format: String,
    pub priority: String,
    pub tags: Vec<String>,
    pub url: Option<String>,
    pub mentions: Vec<String>,
}

impl SendRequestInput {
    /// Extract known fields from the raw request map, applying the default
    /// policy: strings default to `""`, `format` to `"text"`, `priority` to
    /// `"normal"`, and string arrays to `[]`.
    pub fn from_map(map: &Map<String, Value>) -> Self {
        SendRequestInput {
            title: map
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            body: map
                .get("body")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            format: map
                .get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("text")
                .to_string(),
            priority: map
                .get("priority")
                .and_then(|v| v.as_str())
                .unwrap_or("normal")
                .to_string(),
            tags: string_list(map, "tags"),
            url: map
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            mentions: string_list(map, "mentions"),
        }
    }

    /// Convert into the channel-layer request; `template_id` and `params` are
    /// fixed for the plain send endpoint.
    pub fn into_notification_request(self) -> NotificationRequest {
        NotificationRequest {
            title: self.title,
            body: self.body,
            format: self.format,
            priority: self.priority,
            tags: self.tags,
            url: self.url,
            mentions: self.mentions,
            template_id: None,
            params: Value::Null,
        }
    }
}

/// Cleaned input of `POST /api/notify/template`.
pub struct SendTemplateRequestInput {
    pub template_name: Option<String>,
    pub variables: Map<String, Value>,
}

impl SendTemplateRequestInput {
    /// Extract known fields from the raw request map; `variables` defaults to
    /// an empty object. `template_name` is validated by the handler.
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
