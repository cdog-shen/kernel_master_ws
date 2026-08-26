# 命名规范

## 目的

统一 kernel master 全仓库的命名口径，使 crate、文件、结构体、数据库对象与配置键在
8 个成员 crate 之间保持一致、可预测。命名一致性直接降低跨 crate 阅读与代码生成的
成本，也是 AI 编码助手产出正确代码的前提。本文档区分"必须遵守的规则"与"现状
例外"，新增代码一律按规则执行。

## crate 命名

必须使用 kebab-case。命名分两类：

- 后端服务以 `-backend` 结尾：`watchman-backend`、`cmdb-backend`。
- 工具库与子系统用短名：`share-lib`、`yell`、`file-agent`、`jc-commander`、`jc-worker`。

新增 crate 禁止出现下划线、大写或驼峰命名，并须同步登记根 `Cargo.toml` 的
`members`（见根 `Cargo.toml:2-11`）。

✅

```toml
members = [
    "cloud-api",
    "cmdb-backend",
    "file-agent",
    "jc-commander",
    "jc-worker",
    "share-lib",
    "watchman-backend",
    "yell",
]
```

❌ `watchmanBackend`、`watchman_backend`、`Watchman-Backend`

## 文件命名

Rust 源文件名一律 snake_case。按职责使用后缀：

- HTTP handler 文件必须以 `_manage.rs` 结尾，如
  `watchman-backend/src/api/account_manage.rs`、`cmdb-backend/src/api/table_manage.rs`、
  `file-agent/src/api/dir_manage.rs`。
- service 文件必须以 `_service.rs` 结尾，如
  `watchman-backend/src/service/account_service.rs`、
  `cmdb-backend/src/service/km/job_log_service.rs`。
- 健康检查端点固定为 `hey_hi_hello.rs`（见
  `watchman-backend/src/api/hey_hi_hello.rs`），各服务必须复用此文件名。
- 系统管理端点（如 `/api/reload`）固定为 `system_manage.rs`（见
  `watchman-backend/src/api/system_manage.rs`）。

❌ `account_handler.rs`、`AccountManage.rs`、`health.rs`、`sys.rs`

## Diesel 三结构体模式

每个数据表在 model 层必须定义三个配套结构体，命名以表实体名加固定后缀：

- `XxxModel`：全字段，`#[derive(Queryable, Selectable, Insertable)]`，映射整行。
- `XxxInputStream`：字段全为 `Option<T>`，`#[derive(..., Insertable, AsChangeset)]`，
  用于插入与局部更新，提供 `from_map(Map<String, Value>)` 构造方法。
- `XxxOutputStream`：字段全为 `Option<T>`，必须剔除敏感字段（如 `passwd`），
  提供 `from_model` 转换方法。

✅（摘自 `watchman-backend/src/model/user.rs:12-46`）

```rust
#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = user_table)]
pub struct UserModel {
    pub id: i32,
    pub username: String,
    pub passwd: String,
    // ...
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = user_table)]
pub struct UserInputStream {
    pub id: Option<i32>,
    pub username: Option<String>,
    // ...
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserOutputStream {
    pub id: Option<i32>,
    pub username: Option<String>,
    // 注意：不含 passwd
    // ...
}
```

❌ 把 `passwd` 留在 `OutputStream`；给输入结构体起 `UserIn` / `UserReq` 等自创后缀。

## MailMan 命名体系

统一消息结构以 `MailMan` 为前缀：`MailManOk`、`MailManErr`
（`share-lib/src/data_structure.rs:7,20`）、`MailManErrResponser`
（`share-lib/src/err_mapping.rs:15`）。新增的统一消息结构（信封、响应体等）
必须沿用 `MailMan` 前缀，禁止另起 `ResponseOk`、`ApiResult` 之类的平行体系。

✅ `MailManOk::new(200, "config load DONE", None::<&str>)`
（见 `watchman-backend/src/api/system_manage.rs`）

## 数据库命名

- 表名必须使用 snake_case 并以 `_table` 结尾，如 `user_table`、`access_table`
  （见 `watchman-backend/src/model/schema.rs`）。
- 列名 snake_case，与 model 字段一一对应。
- `schema.rs` 由 diesel CLI 自动生成，禁止手工修改；表结构变更走
  `migrations/` 的 `up.sql` / `down.sql` 后重新生成。

❌ `CREATE TABLE users (...)`（缺后缀、复数名）

## 配置键命名

`*.template.toml` 中的配置键必须使用 snake_case，如
`watchman-backend/watchman.template.toml` 中的 `log_path`、`listen_addr`、
`listen_port`、`authenticate_bypass`。分组使用 `[xxx_config]` 形式
（`[server_config]`、`[db_config]`）。新增配置键禁止用 kebab-case 或驼峰。

✅

```toml
[server_config]
listen_addr = "0.0.0.0"
listen_port = 8000
```

❌ `listenAddr`、`listen-addr`

## Rust 通用命名

遵循 Rust 官方惯例，全仓库统一：

- 函数、变量、模块、文件：snake_case，如 `get_user_by_username`、
  `reload_config`。
- 类型、结构体、枚举、trait：CamelCase，如 `UserModel`、`MailManErr`。
- 常量与静态量：SCREAMING_SNAKE_CASE，如 `BAD_REQUEST_CODE`。
- Cargo feature 名：kebab-case，如 `cmdb-backend/Cargo.toml` 的
  `full-schema`、`share-lib/Cargo.toml` 的 `web`。

## 拼写质量

标识符与注释中的英文必须拼写正确。提交前通读一遍新增标识符；不确定的单词查
词典，不要凭印象拼。历史遗留的拼写错误如涉及外部兼容（数据库列名、消息字段、
公开 API），允许保留，但新增代码不得效仿。

## 现状与例外

- `UNKNOW_ERROR_CODE`（应为 `UNKNOWN_ERROR_CODE`）存在于多个 crate 的 model 层
  （如 `watchman-backend/src/model/user.rs:8`），因其仅作内部错误码使用且牵涉面广，
  暂不统一重命名；新代码必须写成 `UNKNOWN_ERROR_CODE` 或改用 MailMan 体系。
- 同类问题还有错误字符串 `"Unknow Error"`（同文件多处），同样按"存量保留、
  新增不效仿"处理。
- 历史上还出现过 `serializtion`、`oulook` 等拼写错误，均已修正；仓库现有代码中
  已不存在这两处错误，作为反面案例记录。
- 表名 `_table` 后缀目前只有 `watchman-backend` 的表完全遵守；`cmdb-backend` 的
  `cloud_account`、`job_log` 等表与 `yell` 的 `notification_records` 等表未带后缀，
  属于早期历史遗留。改名会破坏既有数据库与迁移脚本，本期不动；**新增表必须带
  `_table` 后缀**。
- `file-agent/src/service/` 下的 `dir_manage.rs`、`file_manage.rs` 使用 `_manage`
  后缀而非 `_service`，是 file-agent 无数据库、service 层直接对应文件操作的
  历史写法，暂不统一。
- `jc-commander/src/api/` 的 `cron_job.rs`、`job_log.rs` 与 `yell/src/api/` 的
  `notify.rs` 未带 `_manage` 后缀，新增 handler 文件不得效仿。
- `yell/src/api/` 目前缺少 `system_manage.rs`，后续补充时必须使用该文件名。
