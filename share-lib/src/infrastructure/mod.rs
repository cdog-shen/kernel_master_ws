//! 非 DB 原子操作层
//!
//! 编排层（service）需要的非数据库原子操作统一收敛在本模块：
//! - `http_client`：HTTP 外呼（feature `http`，基于 ureq）
//! - `mq_client`：消息队列操作（feature `mq`，基于 lapin）
//! - `process_runner`：本地进程派生（常驻，基于 std::process）
//!
//! DB 原子操作仍归各 crate 的 `model/` 目录，不在本层范围内。

#[cfg(feature = "http")]
pub mod http_client;
#[cfg(feature = "mq")]
pub mod mq_client;
pub mod process_runner;
