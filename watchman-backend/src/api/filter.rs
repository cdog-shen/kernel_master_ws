//! Cleaning-layer helpers for GET query filters.
//!
//! Each table has a whitelist of filter keys (mirroring the keys supported by
//! the model's `get_*_with_filter`) together with the expected value type.
//! Unknown keys and wrongly-typed values are stripped before the filter map is
//! passed down; this is semantically equivalent to the model's silent
//! ignoring, so no 400 error is introduced.

use serde_json::{Map, Value};

/// Expected JSON value type for a whitelisted filter key.
#[derive(Clone, Copy)]
enum FilterValueType {
    Str,
    Bool,
    Int,
}

impl FilterValueType {
    fn matches(self, value: &Value) -> bool {
        match self {
            FilterValueType::Str => value.is_string(),
            FilterValueType::Bool => value.is_boolean(),
            FilterValueType::Int => value.as_i64().is_some(),
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
