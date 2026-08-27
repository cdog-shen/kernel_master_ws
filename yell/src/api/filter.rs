//! Cleaning-layer helpers for query filters and send requests.
//!
//! Filter whitelists mirror the keys supported by the model's
//! `get_with_filter` together with the expected value type; unknown keys and
//! wrongly-typed values are stripped before the filter map is passed down,
//! which is semantically equivalent to the model's silent ignoring, so no 400
//! error is introduced.
//!
//! The send-request input struct replaces manual per-field extraction in the
//! handlers; its `from_map` constructor applies the exact same default
//! policy the handlers used before.

use serde_json::{Map, Value};

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
