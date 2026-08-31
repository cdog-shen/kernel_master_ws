//! HTTP 外呼原子操作（基于 ureq 的同步实现）
//!
//! 统一约定：
//! - 成功返回响应体文本（`String`）；
//! - 非 2xx 响应视为错误，错误 msg 中携带状态码与目标 URL；
//! - 网络层失败（连接失败、超时等）同样转为 `MailManErr`；
//! - query 参数经由 ureq `.query()` 拼接，自动 percent-encode；
//! - 全局 Agent 带默认超时（连接 3s、整体 5s），防止对端挂起拖垮调用线程；
//! - `*_with_status` 变体返回结构化 `(状态码, 响应体)`，供需要状态码分流的调用方使用。

use std::sync::LazyLock;
use std::time::Duration;

use crate::data_structure::MailManErr;

/// 全局 HTTP Agent：连接超时 3s、整体超时 5s；
/// 关闭“非 2xx 转 Error”（`http_status_as_error(false)`），
/// 状态码改由核心函数统一处理，避免结构化信息丢失
static HTTP_AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
    let config = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(3)))
        .timeout_global(Some(Duration::from_secs(5)))
        .http_status_as_error(false)
        .build();
    ureq::Agent::new_with_config(config)
});

/// GET 请求，返回响应体文本
pub fn get(
    url: &str,
    headers: &[(String, String)],
    query: &[(String, String)],
) -> Result<String, MailManErr<'static, String>> {
    let mut req = HTTP_AGENT.get(url);
    for (key, value) in headers {
        req = req.header(key, value);
    }
    for (key, value) in query {
        req = req.query(key, value);
    }
    handle_response(req.call(), url, "Infrastructure: HTTP GET")
}

/// DELETE 请求，返回响应体文本
pub fn delete(
    url: &str,
    headers: &[(String, String)],
    query: &[(String, String)],
) -> Result<String, MailManErr<'static, String>> {
    let mut req = HTTP_AGENT.delete(url);
    for (key, value) in headers {
        req = req.header(key, value);
    }
    for (key, value) in query {
        req = req.query(key, value);
    }
    handle_response(req.call(), url, "Infrastructure: HTTP DELETE")
}

/// POST 请求，body 为 JSON，返回响应体文本
pub fn post_json(
    url: &str,
    headers: &[(String, String)],
    query: &[(String, String)],
    body: &serde_json::Value,
) -> Result<String, MailManErr<'static, String>> {
    let mut req = HTTP_AGENT.post(url);
    for (key, value) in headers {
        req = req.header(key, value);
    }
    for (key, value) in query {
        req = req.query(key, value);
    }
    handle_response(req.send_json(body), url, "Infrastructure: HTTP POST")
}

/// POST 请求（body 为 JSON），返回结构化结果 `(HTTP 状态码, 响应体)`
///
/// 与 `post_json` 不同：非 2xx 不视为错误（由调用方按状态码分流），
/// 仅网络层失败（连接失败、超时等）返回 `Err`
pub fn post_json_with_status(
    url: &str,
    headers: &[(String, String)],
    query: &[(String, String)],
    body: &serde_json::Value,
) -> Result<(u16, String), MailManErr<'static, String>> {
    let mut req = HTTP_AGENT.post(url);
    for (key, value) in headers {
        req = req.header(key, value);
    }
    for (key, value) in query {
        req = req.query(key, value);
    }
    send_and_read(req.send_json(body), url, "Infrastructure: HTTP POST")
}

/// POST 请求，body 为原始字符串（Content-Type 由 headers 指定），返回响应体文本
pub fn post_raw(
    url: &str,
    headers: &[(String, String)],
    query: &[(String, String)],
    body: &str,
) -> Result<String, MailManErr<'static, String>> {
    let mut req = HTTP_AGENT.post(url);
    for (key, value) in headers {
        req = req.header(key, value);
    }
    for (key, value) in query {
        req = req.query(key, value);
    }
    handle_response(req.send(body), url, "Infrastructure: HTTP POST")
}

/// PUT 请求，body 为 JSON，返回响应体文本
pub fn put_json(
    url: &str,
    headers: &[(String, String)],
    query: &[(String, String)],
    body: &serde_json::Value,
) -> Result<String, MailManErr<'static, String>> {
    let mut req = HTTP_AGENT.put(url);
    for (key, value) in headers {
        req = req.header(key, value);
    }
    for (key, value) in query {
        req = req.query(key, value);
    }
    handle_response(req.send_json(body), url, "Infrastructure: HTTP PUT")
}

/// 核心函数：读取响应为结构化 `(HTTP 状态码, 响应体)`
///
/// 仅网络层失败（连接失败、超时等）返回 `Err`；
/// 由于全局 Agent 关闭了非 2xx 转 Error，HTTP 层面的响应（含 4xx/5xx）都走 `Ok`
fn send_and_read(
    result: Result<ureq::http::Response<ureq::Body>, ureq::Error>,
    url: &str,
    key: &'static str,
) -> Result<(u16, String), MailManErr<'static, String>> {
    match result {
        Ok(mut resp) => {
            let status = resp.status().as_u16();
            let body = resp.body_mut().read_to_string().map_err(|e| {
                MailManErr::new(500, key, Some(format!("读取 {url} 响应体失败: {e}")), 1)
            })?;
            Ok((status, body))
        }
        Err(e) => Err(MailManErr::new(
            500,
            key,
            Some(format!("请求 {url} 失败: {e}")),
            1,
        )),
    }
}

/// 统一处理 ureq 响应：读取响应体文本，非 2xx 视为错误
fn handle_response(
    result: Result<ureq::http::Response<ureq::Body>, ureq::Error>,
    url: &str,
    key: &'static str,
) -> Result<String, MailManErr<'static, String>> {
    let (status, body) = send_and_read(result, url, key)?;
    if (200..300).contains(&status) {
        Ok(body)
    } else {
        Err(MailManErr::new(
            500,
            key,
            Some(format!("请求 {url} 返回非 2xx 状态码: {status}")),
            1,
        ))
    }
}
