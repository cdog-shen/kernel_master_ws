//! Cleaning-layer helper for GET query filters.
//!
//! The whitelist mirrors the keys supported by
//! `CloudAccountModel::get_model_with_filter` together with the expected value
//! type. Unknown keys and wrongly-typed values are stripped before the filter
//! map is passed down; this is semantically equivalent to the model's silent
//! ignoring, so no 400 error is introduced.

use serde_json::{Map, Value};

/// Whitelist for `cloud_account`, see `CloudAccountModel::get_model_with_filter`.
const CLOUD_ACCOUNT_FILTER_KEYS: &[&str] = &["provider", "nick_name"];

/// Clean the filter for the `get_all` endpoint. All whitelisted keys expect
/// string values.
pub fn clean_cloud_account_filter(filter: Map<String, Value>) -> Map<String, Value> {
    filter
        .into_iter()
        .filter(|(key, value)| {
            CLOUD_ACCOUNT_FILTER_KEYS.contains(&key.as_str()) && value.is_string()
        })
        .collect()
}
