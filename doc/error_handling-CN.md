# 错误处理规范

## 目的

本规范统一各 crate 的错误表达方式：所有错误必须经过 MailMan 消息体系流转，
最终由 `MailManErrResponser` 映射为 HTTP 响应。目标是让错误码语义可预测、
日志不缺失、公共代码不重复。阅读对象：项目开发者与 AI 编码助手。

## MailMan 消息体系

成功与失败消息统一定义在 `share-lib/src/data_structure.rs`：

```rust
pub struct MailManOk<'a, T> {
    pub code: u8,
    pub key: &'a str,
    pub data: Option<T>,
}

pub struct MailManErr<'a, T> {
    pub code: u16,
    pub key: &'a str,
    pub msg: Option<T>,
    pub level: u8, // 0-2 as warn error fatal
}
```

两个构造函数都带有日志副作用：`MailManOk::new` 内部调用 `log_info!("{}", key)`；
`MailManErr::new` 按 `level` 分发日志级别——`0` 记 warn，`1` 记 error，
其余值记 trace（注意字段注释写的是 "fatal"，与实现不一致，以实现为准）：

```rust
match level {
    0 => log_warn!("{}, {:?}", key, error_obj),
    1 => log_error!("{}, {:?}", key, error_obj),
    _ => log_trace!("{}, {:?}", key, error_obj),
};
```

规约：

- 必须利用该副作用记错误日志：构造 `MailManErr` 时一次性确定正确的 `level`，
  不要另行补写日志。
- 禁止为了记日志而构造随后被丢弃的 `MailManOk`。要记 info 日志就直接调用
  `log_info!` 宏。该做法在现有代码中存在（见下文"现状与例外"），新代码不得沿用。

## 三层错误流

错误沿 model → service → api 三层向上传递，每层形态不同。

### model 层：返回 `Result<T, (u8, String)>`

model 层不感知 HTTP，用 `(错误码, 错误消息)` 元组表达错误。错误码以常量定义在
model 文件顶部，见 `watchman-backend/src/model/user.rs`：

```rust
static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

// ...
Err(NotFound) => Err((BAD_REQUEST_CODE, format!("NotFound {user_name}."))),
Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
```

约定：`0` 表示未知/内部错误（`UNKNOW_ERROR_CODE`，常量名中的 "UNKNOW"
为历史拼写，保持一致勿改），`1` 表示请求类错误（`BAD_REQUEST_CODE`）。

### service 层：返回 `Result<MailManOk<'a, T>, MailManErr<'a, String>>`

service 层负责把 model 层的 `(u8, String)` 翻译成带 HTTP 语义的 `MailManErr`，
`key` 按 `"Service: <接口名> [- <步骤>]"` 惯例命名。见
`watchman-backend/src/service/account_service.rs`：

```rust
match TokenModel::delete(&user_info.username.unwrap(), &mut pool.get().unwrap()) {
    Ok(msg) => Ok(MailManOk::new(200, "Service: Logout", Some(msg))),
    Err(msg) => match msg.0 {
        1 => Err(MailManErr::new(400, "Service: Logout", Some(msg.1), 1)),
        _ => Err(MailManErr::new(500, "Service: Logout", Some(msg.1), 1)),
    },
}
```

规约：

- 必须在这一层完成 model 错误码到 HTTP 错误码的翻译：`1` → `400`，
  `0`（及其他未知值）→ `500`。
- `MailManErr::new` 的第 4 个参数（level）：业务上需要报警的错误用 `1`（error），
  预期内的失败（如登录密码错误）可以用 `0`（warn）。

### api 层：用 `MailManErrResponser::mapping_from_mme` 转 HTTP

api handler 返回 `Result<HttpResponse, MailManErrResponser>`，
service 的错误经 `mapping_from_mme` 转换后由 actix-web 自动序列化为 JSON 响应。
见 `watchman-backend/src/api/account_manage.rs`：

```rust
match account_service::login(user_info, &pool) {
    Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
    Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
}
```

入参解析等 api 层自身产生的错误，直接构造 `MailManErr` 后立即映射：

```rust
let user_info = user::UserInputStream::from_map(map.0).map_err(|e| {
    MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
})?;
```

## 错误码到 HTTP 语义的映射

映射定义在 `share-lib/src/err_mapping.rs`，固定为：

| code | HTTP 语义 |
|---|---|
| 400 | BAD_REQUEST |
| 401 | UNAUTHORIZED |
| 403 | FORBIDDEN |
| 404 | NOT_FOUND |
| 501 | NOT_IMPLEMENTED |
| 503 | SERVICE_UNAVAILABLE |
| 其余 | INTERNAL_SERVER_ERROR (500) |

注意：该文件中有两处 `match` 维护同一张表——`mapping_from_mme` 里决定响应
JSON 的 `key` 字段，`ResponseError::status_code()` 决定 HTTP 状态码。
修改映射时两处必须同步，禁止只改一处。

新增错误时必须优先复用上表中的 code；只有现有语义确实无法表达时才引入新 code，
且引入时必须同步更新两处 `match` 与本文档。

## MailManErrResponser 的位置与引用方式

`MailManErrResponser` 收敛在 share-lib，仅在 `web` feature 下编译
（见 `share-lib/Cargo.toml`：`web = ["dep:actix-web"]`）。需要 web 能力的
crate 必须这样引入：

```toml
share-lib = { workspace = true, features = ["web"] }
```

```rust
use share_lib::err_mapping::MailManErrResponser;
```

历史上 `err_mapping.rs` 曾在 6 个 crate 中逐字节复制，现已收敛。
禁止以任何理由在各 crate 内复制该文件或其变体；确需调整映射行为时，
只能修改 `share-lib/src/err_mapping.rs` 并同步全部调用方。

## 字段填写规约

- `code`：优先复用语义映射表中的值，见上节。
- `key`：用人类可读的固定短语，标明出错位置与环节，沿用现有惯例
  （service 层 `"Service: Login - Auth"`，api 层 `"Bad Request"`）。
  禁止把动态数据拼进 `key`——动态信息属于 `msg`。
- `msg`：携带上下文细节（哪条记录、哪个字段、底层错误文本），
  例如 `format!("NotFound {user_name}.")`。可以为 `None`，映射时会兜底为
  `"No more Msg"`，但推荐尽量给出。

## 现状与例外

以下差距如实记录，新代码不得沿用，存量代码逐步整改：

- `MailManErr` 字段注释写 `0-2 as warn error fatal`，但实现中"其余值"记的是
  trace 而非 fatal（`share-lib/src/data_structure.rs`）。以代码行为为准。
- model 层常量名 `UNKNOW_ERROR_CODE` 是 "UNKNOWN" 的拼写错误，为保持兼容不改名。
- `watchman-backend/src/service/account_service.rs` 的 `login` 中存在
  "构造后丢弃 `MailManOk` 仅借其副作用记日志"的写法（更新登录时间、保存 token
  两处），违反本文第二节规约，属历史遗留。
- 各子系统 crate 的本地 `middleware/auth_middleware.rs` 已删除，鉴权失败（401/403/503）
  的短路返回统一由 share-lib `middleware/user_auth.rs` 产生——body 为 `MailManErr`
  序列化 JSON，code → HTTP 状态码的映射在中间件内部自带（不经过
  `MailManErrResponser`）。仅 watchman-backend 保留本地 `JwtAuth`/`PermissionCheck`。
