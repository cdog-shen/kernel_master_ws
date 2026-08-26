//! 向 commander 回报任务结果的 HTTP 外呼原子操作
//!
//! 原为 `service/json_rpc.rs`，实为对 commander `/api/log/update` 的 HTTP 回调，
//! 属 crate 私有的原子操作，按骨架规约归位 `util/`，
//! 基于 share-lib `http_client`（feature `http`）实现。

use chrono::Local;

use share_lib::data_structure::MailManErr;
use share_lib::infrastructure::http_client;

use crate::config::worker;

/// 回调 commander 更新任务日志，成功返回响应体文本
pub fn update_log(
    auth: &str,
    uuid: &str,
    status: u8,
    result: &str,
) -> Result<String, MailManErr<'static, String>> {
    let commander_url = format!(
        "http://{}:{}/api/log/update",
        worker::GLOBAL_CONFIG.read().unwrap().commander_addr,
        worker::GLOBAL_CONFIG.read().unwrap().commander_port
    );
    let worker_id = worker::GLOBAL_CONFIG.read().unwrap().subsys_uuid.clone();
    let headers = [
        ("Content-Type".to_string(), "application/json".to_string()),
        ("Authorization".to_string(), format!("uuid {auth}")),
        ("Connection".to_string(), "close".to_string()),
    ];
    let body = serde_json::json!({
        "id": uuid,
        "worker": worker_id,
        "status": status,
        "result": result,
        "finish_time": Local::now().naive_local().format("%Y-%m-%dT%H:%M:%S").to_string(),
    });

    http_client::post_json(&commander_url, &headers, &[], &body)
}
