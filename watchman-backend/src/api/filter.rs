//! Cleaning-layer helpers for GET query filters.
//!
//! Each table has a whitelist of filter keys (mirroring the keys supported by
//! the model's `get_*_with_filter`) together with the expected value type.
//! GET query parameters always parse to strings, so cleaning coerces string
//! values of whitelisted bool/int keys to their native type. Unknown keys and
//! wrongly-typed values (including failed coercions) are stripped before the
//! filter map is passed down; this is semantically equivalent to the model's
//! silent ignoring, so no 400 error is introduced.

use serde_json::{Map, Value};

/// Expected JSON value type for a whitelisted filter key.
#[derive(Clone, Copy)]
enum FilterValueType {
    Str,
    Bool,
    Int,
}

impl FilterValueType {
    /// Normalize a value to the expected type: native-typed values pass
    /// through; string values (GET query parameters always parse to strings)
    /// are coerced to the target type, i.e. "true"/"false" -> bool and
    /// numeric strings -> i64; a failed coercion or type mismatch returns
    /// None and the key is stripped (preserving the silent-ignore semantics).
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

/// Strip keys outside the whitelist and coerce string-form bool/int values to
/// their native type; wrongly-typed values and failed coercions are stripped.
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

/// Whitelist for `user_table`, see `UserModel::get_user_info_with_filter`.
const USER_FILTER_WHITELIST: &[(&str, FilterValueType)] = &[
    ("username", FilterValueType::Str),
    ("is_enable", FilterValueType::Bool),
    ("full_name", FilterValueType::Str),
];

/// Whitelist for `group_table`, see `GroupModel::get_all_with_filter`.
const GROUP_FILTER_WHITELIST: &[(&str, FilterValueType)] = &[
    ("group_name", FilterValueType::Str),
    ("is_enable", FilterValueType::Bool),
    ("user_id", FilterValueType::Int),
];

/// Whitelist for `access_table`, see `AccessModel::get_all_with_filter`.
const ACCESS_FILTER_WHITELIST: &[(&str, FilterValueType)] = &[
    ("service_id", FilterValueType::Int),
    ("group_id", FilterValueType::Int),
    ("group_access", FilterValueType::Int),
    ("is_enable", FilterValueType::Bool),
    ("comment", FilterValueType::Str),
];

/// Whitelist for `service_table`, see `ServiceModel::get_all_with_filter`.
const SERVICE_FILTER_WHITELIST: &[(&str, FilterValueType)] = &[
    ("service_name", FilterValueType::Str),
    ("is_enable", FilterValueType::Bool),
    ("service_point", FilterValueType::Str),
];

/// Whitelist for `subsystem_table`, see `SubsysModel::get_all_with_filter`.
const SUBSYS_FILTER_WHITELIST: &[(&str, FilterValueType)] = &[
    ("subsys_name", FilterValueType::Str),
    ("is_enable", FilterValueType::Bool),
    ("url", FilterValueType::Str),
    ("relate_service_id", FilterValueType::Int),
];

/// Whitelist for `webhook_table`, see `WebhookModel::get_all_with_filter`.
const WEBHOOK_FILTER_WHITELIST: &[(&str, FilterValueType)] = &[
    ("hook_name", FilterValueType::Str),
    ("is_enable", FilterValueType::Bool),
];

/// Clean the filter for the `all_user` endpoint.
pub fn clean_user_filter(filter: Map<String, Value>) -> Map<String, Value> {
    clean_filter(filter, USER_FILTER_WHITELIST)
}

/// Clean the filter for the `all_group` endpoint.
pub fn clean_group_filter(filter: Map<String, Value>) -> Map<String, Value> {
    clean_filter(filter, GROUP_FILTER_WHITELIST)
}

/// Clean the filter for the `all_access` endpoint.
pub fn clean_access_filter(filter: Map<String, Value>) -> Map<String, Value> {
    clean_filter(filter, ACCESS_FILTER_WHITELIST)
}

/// Clean the filter for the `all_service` endpoint.
pub fn clean_service_filter(filter: Map<String, Value>) -> Map<String, Value> {
    clean_filter(filter, SERVICE_FILTER_WHITELIST)
}

/// Clean the filter for the `all_subsys` endpoint.
pub fn clean_subsys_filter(filter: Map<String, Value>) -> Map<String, Value> {
    clean_filter(filter, SUBSYS_FILTER_WHITELIST)
}

/// Clean the filter for the `all_webhook` endpoint.
pub fn clean_webhook_filter(filter: Map<String, Value>) -> Map<String, Value> {
    clean_filter(filter, WEBHOOK_FILTER_WHITELIST)
}
