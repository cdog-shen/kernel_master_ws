//! HTTP 外呼原子操作（基于 ureq 的同步实现）
//!
//! 统一约定：
//! - 成功返回响应体文本（`String`）；
//! - 非 2xx 响应视为错误，错误 msg 中携带状态码与目标 URL；
//! - 网络层失败（连接失败、超时等）同样转为 `MailManErr`；
//! - query 参数经由 ureq `.query()` 拼接，自动 percent-encode。

use crate::data_structure::MailManErr;

/// GET 请求，返回响应体文本
pub fn get(
    url: &str,
    headers: &[(String, String)],
    query: &[(String, String)],
) -> Result<String, MailManErr<'static, String>> {
    let mut req = ureq::get(url);
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
    let mut req = ureq::delete(url);
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
    let mut req = ureq::post(url);
    for (key, value) in headers {
        req = req.header(key, value);
    }
    for (key, value) in query {
        req = req.query(key, value);
    }
    handle_response(req.send_json(body), url, "Infrastructure: HTTP POST")
}

/// POST 请求，body 为原始字符串（Content-Type 由 headers 指定），返回响应体文本
pub fn post_raw(
    url: &str,
    headers: &[(String, String)],
    query: &[(String, String)],
    body: &str,
) -> Result<String, MailManErr<'static, String>> {
    let mut req = ureq::post(url);
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
    let mut req = ureq::put(url);
    for (key, value) in headers {
        req = req.header(key, value);
    }
    for (key, value) in query {
        req = req.query(key, value);
    }
    handle_response(req.send_json(body), url, "Infrastructure: HTTP PUT")
}

/// 统一处理 ureq 响应：读取响应体文本，非 2xx 视为错误
fn handle_response(
    result: Result<ureq::http::Response<ureq::Body>, ureq::Error>,
    url: &str,
    key: &'static str,
) -> Result<String, MailManErr<'static, String>> {
    match result {
        Ok(mut resp) => resp
            .body_mut()
            .read_to_string()
            .map_err(|e| MailManErr::new(500, key, Some(format!("读取 {url} 响应体失败: {e}")), 1)),
        Err(ureq::Error::StatusCode(code)) => Err(MailManErr::new(
            500,
            key,
            Some(format!("请求 {url} 返回非 2xx 状态码: {code}")),
            1,
        )),
        Err(e) => Err(MailManErr::new(
            500,
            key,
            Some(format!("请求 {url} 失败: {e}")),
            1,
        )),
    }
}
