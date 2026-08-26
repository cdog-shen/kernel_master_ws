# 日志规范

## 目的

统一各 crate 的日志写法：用什么设施、在哪里初始化、什么场景用什么级别、
以及 MailMan 体系与日志的关系。目标是同一条信息只被记录一次、格式一致、
可通过配置控制级别，避免重复日志和随手 `println!` 造成的噪音。

## 日志设施

facade 使用 `log` crate，实现是 share-lib 自研的 `ShareLogger`
（`share-lib/src/logger.rs`）。它持有一个 `Arc<Mutex<File>>` 以追加模式写日志文件，
同时用 `println!` 同步输出到控制台；每条日志的格式为
`{rfc3339 时间戳} - {LEVEL} - {target} - {msg}`（`share-lib/src/logger.rs:53`）。

业务代码必须调用 share-lib 导出的宏，禁止直接使用 `println!`/`eprintln!` 记录业务日志：

```rust
// share-lib/src/logger.rs
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => (info!($($arg)*));
}
// 另有 log_warn! / log_error! / log_debug! / log_trace!，一一对应 log crate 宏
```

注意：这些宏展开为不带路径的 `info!`/`warn!` 等，调用模块必须同时
`use log::{info, warn, error, debug, trace};`（用到哪个导入哪个），
参考 `share-lib/src/data_structure.rs:1-2` 的写法。

✅ 推荐：

```rust
use share_lib::log_info;
use log::info;

log_info!("DB Pool init");
```

❌ 禁止：

```rust
println!("user {} login ok", username); // 绕过日志级别控制，无法落盘分级
```

## 初始化

每个二进制 crate 必须在 `main` 函数开头、完成配置加载之后调用
`logger::init_logger(log_path, log_level)`，两个参数来自 `GLOBAL_CONFIG`。
`init_logger` 内部完成 `log::set_boxed_logger` 与 `log::set_max_level`，
只能调用一次（重复调用会 panic）。参考 `watchman-backend/src/main.rs:44-48`：

```rust
// watchman-backend/src/main.rs
// init logger
logger::init_logger(
    &server::GLOBAL_CONFIG.read().unwrap().log_path,
    &server::GLOBAL_CONFIG.read().unwrap().log_level,
);
```

`log_path` 与 `log_level` 在各 crate 的 `*.template.toml` 中配置，
`log_level` 取值为 `ERROR` / `WARN` / `INFO` / `DEBUG`（其余值按 `Trace` 处理，
见 `share-lib/src/logger.rs:24-30`）。

## 级别约定

级别必须按语义选择，禁止为了"让日志更显眼"而调高：

- `log_error!` — 操作失败，需要人工介入（写库失败、外部调用报错且无法降级）。
- `log_warn!` — 可恢复的异常（重试成功前的失败、配置缺失但走了默认值）。
- `log_info!` — 关键业务节点（服务启动、配置重载完成、登录成功），低频。
- `log_debug!` / `log_trace!` — 诊断细节（请求参数、中间变量），可以高频，
  生产环境通过 `log_level` 关闭。

❌ 禁止：在循环或高频路径中使用 `info` 及以上级别输出逐条数据。

## HTTP 访问日志

HTTP 访问日志（请求方法、路径、状态码、耗时）统一由
`actix_web::middleware::Logger` 输出，各 crate 在 `HttpServer` 构建处注册
（见 `watchman-backend/src/main.rs:91`）：

```rust
// watchman-backend/src/main.rs
// wrap default logger
.wrap(actix_web::middleware::Logger::default())
```

handler 内禁止再手工记录请求行（方法 + path 的 info 日志），避免与
middleware 输出重复。handler 内只记录业务事件本身。

## MailMan 与日志的关系

`MailManOk::new` / `MailManErr::new` 在构造时有记日志的副作用
（`share-lib/src/data_structure.rs`）：

```rust
// share-lib/src/data_structure.rs
pub fn new(code: u16, key: &'a str, error_obj: Option<T>, level: u8) -> Self {
    match level {
        0 => log_warn!("{}, {:?}", key, error_obj),
        1 => log_error!("{}, {:?}", key, error_obj),
        _ => log_trace!("{}, {:?}", key, error_obj),
    };
    // ...
}
```

由此约定（必须遵守）：

1. 错误必须在其产生处（最底层拿到错误对象的位置）构造 `MailManErr`，
   构造即完成记日志；调用链上层只做 `Err(...)` 透传，禁止每层重复记录同一错误。
2. 普通的过程日志使用 `log_*!` 宏，不借助 MailMan。
3. 禁止为了记日志而构造并丢弃 `MailManOk`——成功路径的过程信息用 `log_info!`。

✅ 推荐：

```rust
// 底层：错误产生处构造，记一次日志
Err(msg) => Err(MailManErr::new(500, "Service: Login", Some(msg.1), 1)),
```

```rust
// 上层：仅透传，不再记
let user = login(req, pool)?; // 或 match 后原样 return Err(e)
```

❌ 禁止：

```rust
// watchman-backend/src/service/account_service.rs:82-86（现状反例，待收敛）
Ok(msg) => {
    MailManOk::new(            // 构造后丢弃返回值，仅为触发其 info 日志副作用
        200,
        "Service: Login - login time",
        Some(format!("Line changed: {msg}")),
    );
}
```

上述场景应改为 `log_info!("Service: Login - login time, Line changed: {msg}")`。

## 现状与例外

- 本仓库没有引入 `tracing`、`metrics`、`env_logger`、`log4rs`，
  全部日志走 `log` facade + `ShareLogger`，暂无结构化字段与指标采集能力；
  若未来引入 `tracing`，需先评估与 `ShareLogger` 及 MailMan 副作用的兼容方案。
- `ShareLogger` 的 `flush` 是空实现（`share-lib/src/logger.rs:70`），
  进程崩溃时理论上可能丢失尚未刷盘的行；当前接受此风险，暂不改造。
- `MailManOk::new` 也会以 info 级别记录 key（`share-lib/src/data_structure.rs:14`），
  即每个成功响应都会留下一条 info 日志，这是既有行为，本期不改动；
  编写日志审查脚本时注意区分 MailMan 产生的 key 日志与业务 `log_info!`。
- 历史代码中存在构造后丢弃 `MailManOk` 的记日志写法
  （如 `watchman-backend/src/service/account_service.rs:82`、`:101`、`:107`），
  按本期规约暂不批量修改，新增代码必须遵守本文"MailMan 与日志的关系"一节。
- `watchman-backend/src/main.rs:42` 在 logger 初始化前用 `println!` 输出全量配置，
  属于启动期调试遗留，允许保留，但新增代码不得效仿。
