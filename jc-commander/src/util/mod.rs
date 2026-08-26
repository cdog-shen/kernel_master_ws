pub mod scheduler;

// MQ 队列名拼装统一入口。
//
// 全 crate 只允许通过这两个函数获取 `{prefix}_sync` / `{prefix}_async`
// 队列名（prefix 取自 GLOBAL_CONFIG.mq_queue_prefix），
// 避免多处各自 `format!` 拼装导致生产/消费两端队列名漂移。

/// sync 任务队列名：`{prefix}_sync`
pub fn mq_sync_queue() -> String {
    let prefix = crate::config::server::GLOBAL_CONFIG
        .read()
        .unwrap()
        .mq_queue_prefix
        .clone();
    format!("{prefix}_sync")
}

/// async 任务队列名：`{prefix}_async`
pub fn mq_async_queue() -> String {
    let prefix = crate::config::server::GLOBAL_CONFIG
        .read()
        .unwrap()
        .mq_queue_prefix
        .clone();
    format!("{prefix}_async")
}
