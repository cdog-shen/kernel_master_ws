//! Cleaning-layer helper for the `api/table/query` endpoint.
//!
//! Each table has a whitelist of filter keys (mirroring the keys supported by
//! the model's `get_*_with_filter`) together with the expected value type.
//! Although this endpoint takes a JSON body, string values of whitelisted int
//! keys are still coerced to their native type for robustness. Unknown keys
//! and wrongly-typed values (including failed coercions) are stripped before
//! the filter map is dispatched to the per-table service; this is semantically
//! equivalent to the model's silent ignoring, so no 400 error is introduced.

use serde_json::{Map, Value};

/// Expected JSON value type for a whitelisted filter key.
#[derive(Clone, Copy)]
enum FilterValueType {
    Str,
    Int,
}

impl FilterValueType {
    /// Normalize a value to the expected type: native-typed values pass
    /// through; string values are coerced to the target type, i.e. numeric
    /// strings -> i64; a failed coercion or type mismatch returns None and
    /// the key is stripped (preserving the silent-ignore semantics).
    fn coerce(self, value: Value) -> Option<Value> {
        match self {
            FilterValueType::Str => value.is_string().then_some(value),
            FilterValueType::Int => match value {
                Value::Number(ref n) if n.is_i64() => Some(value),
                Value::String(ref s) => s.parse::<i64>().ok().map(|n| Value::Number(n.into())),
                _ => None,
            },
        }
    }
}

/// Per-table filter whitelists, one entry per table served by `query_table`.
/// Keys mirror the `get_*_with_filter` implementations in `crate::model`.
const TABLE_FILTER_WHITELISTS: &[(&str, &[(&str, FilterValueType)])] = &[
    // === cmdb native tables ===
    (
        "lighthouse",
        &[
            ("provider", FilterValueType::Str),
            ("zone", FilterValueType::Str),
            ("platform", FilterValueType::Str),
            ("instance_id", FilterValueType::Str),
            ("instance_name", FilterValueType::Str),
        ],
    ),
    (
        "cloudserver_instance",
        &[
            ("provider", FilterValueType::Str),
            ("zone", FilterValueType::Str),
            ("platform", FilterValueType::Str),
            ("instance_id", FilterValueType::Str),
            ("status", FilterValueType::Str),
            ("instance_name", FilterValueType::Str),
        ],
    ),
    (
        "logservice_topic",
        &[
            ("provider", FilterValueType::Str),
            ("set_id", FilterValueType::Str),
            ("topic_id", FilterValueType::Str),
            ("topic_name", FilterValueType::Str),
            ("status", FilterValueType::Str),
        ],
    ),
    (
        "cloud_account",
        &[
            ("provider", FilterValueType::Str),
            ("nick_name", FilterValueType::Str),
        ],
    ),
    (
        "job_log",
        &[
            ("exec_type", FilterValueType::Str),
            ("worker", FilterValueType::Str),
        ],
    ),
    (
        "cron_job",
        &[
            ("id", FilterValueType::Str),
            ("script", FilterValueType::Str),
        ],
    ),
    (
        "cloudstorage_bucket",
        &[
            ("provider", FilterValueType::Str),
            ("bucket_name", FilterValueType::Str),
        ],
    ),
    // === watchman tables (dispatch arms are feature-gated) ===
    (
        "access_table",
        &[
            ("service_id", FilterValueType::Int),
            ("group_id", FilterValueType::Int),
        ],
    ),
    ("group_table", &[("group_name", FilterValueType::Str)]),
    (
        "service_table",
        &[
            ("service_name", FilterValueType::Str),
            ("service_point", FilterValueType::Str),
        ],
    ),
    (
        "subsystem_table",
        &[
            ("subsys_name", FilterValueType::Str),
            ("url", FilterValueType::Str),
        ],
    ),
    ("token_table", &[("username", FilterValueType::Str)]),
    (
        "user_table",
        &[
            ("username", FilterValueType::Str),
            ("full_name", FilterValueType::Str),
        ],
    ),
    (
        "webhook_table",
        &[
            ("hook_name", FilterValueType::Str),
            ("method_type", FilterValueType::Str),
        ],
    ),
    // === yell tables (dispatch arms are feature-gated) ===
    ("channel_configs", &[("channel_type", FilterValueType::Str)]),
    (
        "notification_records",
        &[
            ("channel_type", FilterValueType::Str),
            ("status", FilterValueType::Str),
            ("recipient", FilterValueType::Str),
        ],
    ),
    (
        "notification_templates",
        &[
            ("channel_type", FilterValueType::Str),
            ("name", FilterValueType::Str),
        ],
    ),
    ("notification_groups", &[("name", FilterValueType::Str)]),
    (
        "notification_group_members",
        &[
            ("group_id", FilterValueType::Int),
            ("channel_type", FilterValueType::Str),
        ],
    ),
];

/// Strip filter keys outside the table's whitelist and coerce string-form int
/// values to their native type; wrongly-typed values and failed coercions are
/// stripped. Unknown tables are passed through unchanged (the dispatcher
/// rejects them with 400 on its own).
pub fn clean_query_filter(table_name: &str, query: Map<String, Value>) -> Map<String, Value> {
    let Some((_, whitelist)) = TABLE_FILTER_WHITELISTS
        .iter()
        .find(|(name, _)| *name == table_name)
    else {
        return query;
    };

    query
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
