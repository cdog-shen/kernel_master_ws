//! Cleaning-layer helpers for query filters.
//!
//! Each table has a whitelist of filter keys (mirroring the keys supported by
//! the model's `get_*_with_filter`) together with the expected value type.
//! GET query parameters always parse to strings, so cleaning coerces string
//! values of whitelisted bool keys to their native type. Unknown keys and
//! wrongly-typed values (including failed coercions) are stripped before the
//! filter map is passed down; this is semantically equivalent to the model's
//! silent ignoring, so no 400 error is introduced.

use serde_json::{Map, Value};

/// Expected JSON value type for a whitelisted filter key.
#[derive(Clone, Copy)]
enum FilterValueType {
    Str,
    Bool,
}

impl FilterValueType {
    /// Normalize a value to the expected type: native-typed values pass
    /// through; string values (GET query parameters always parse to strings)
    /// are coerced to the target type, i.e. "true"/"false" -> bool; a failed
    /// coercion or type mismatch returns None and the key is stripped
    /// (preserving the silent-ignore semantics).
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
        }
    }
}

/// Strip keys outside the whitelist and coerce string-form bool values to
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

/// Whitelist for `cron_job`, see `CronJobModel::get_crons_with_filter`.
const CRON_JOB_FILTER_WHITELIST: &[(&str, FilterValueType)] = &[
    ("id", FilterValueType::Str),
    ("script", FilterValueType::Str),
    ("is_enable", FilterValueType::Bool),
];

/// Whitelist for `job_log`, see `JobLogModel::get_logs_with_filter`.
const JOB_LOG_FILTER_WHITELIST: &[(&str, FilterValueType)] = &[
    ("id", FilterValueType::Str),
    ("exec_type", FilterValueType::Str),
    ("worker", FilterValueType::Str),
];

/// Clean the filter for the `cron_job/get_all` endpoint.
pub fn clean_cron_job_filter(filter: Map<String, Value>) -> Map<String, Value> {
    clean_filter(filter, CRON_JOB_FILTER_WHITELIST)
}

/// Clean the filter for the `job_log/get_all` endpoint.
pub fn clean_job_log_filter(filter: Map<String, Value>) -> Map<String, Value> {
    clean_filter(filter, JOB_LOG_FILTER_WHITELIST)
}
