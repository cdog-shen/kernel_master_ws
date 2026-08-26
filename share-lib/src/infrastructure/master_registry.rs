//! 子系统向 master（watchman）刷新注册信息的编排流程
//!
//! 收敛原先散落在各子系统 `api/system_manage.rs` 中的两步外呼：
//! 1. GET  `{base_url}/subsystem_control/all_subsystem?subsys_name=xxx` 查询自身 id；
//! 2. POST `{base_url}/subsystem_control/update_subsystem` 回写 id / subsys_name / token。
//!
//! HTTP 原子操作一律复用 [`crate::infrastructure::http_client`]。

use crate::data_structure::MailManErr;
use crate::infrastructure::http_client;

/// 向 master 刷新本子系统的注册信息（先查 id，再回写 token）
///
/// 参数：
/// - `master_base_url`：master 的 API 前缀，形如 `http://127.0.0.1:8000/api`；
/// - `subsys_name`：本子系统的注册名（`register_name`）；
/// - `token`：本子系统的身份令牌（`subsys_uuid`）。
///
/// 成功时返回 master 回写接口的响应体文本；失败返回 `MailManErr`。
pub fn refresh_master_registration(
    master_base_url: &str,
    subsys_name: &str,
    token: &str,
) -> Result<String, MailManErr<'static, String>> {
    // 第一步：按注册名查询自身在 master 侧的 id
    let query_url =
        format!("{master_base_url}/subsystem_control/all_subsystem?subsys_name={subsys_name}");
    let id_res_body = http_client::get(&query_url, &[], &[])?;

    let json_value: serde_json::Value = serde_json::from_str(&id_res_body).map_err(|e| {
        MailManErr::new(
            500,
            "Infrastructure: master registry",
            Some(format!("解析 {query_url} 响应体为 JSON 失败: {e}")),
            1,
        )
    })?;
    let this_id = json_value["data"][0]["id"].clone();

    // 第二步：回写 id / subsys_name / token
    let req_json = serde_json::json!({
        "id": this_id,
        "subsys_name": subsys_name,
        "token": token,
    });

    http_client::post_json(
        &format!("{master_base_url}/subsystem_control/update_subsystem"),
        &[("Connection".to_string(), "close".to_string())],
        &[],
        &req_json,
    )
}
