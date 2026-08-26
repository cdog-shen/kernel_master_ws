//! Cleaning-layer helpers for query filters.
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
