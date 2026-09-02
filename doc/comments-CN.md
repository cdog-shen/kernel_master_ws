# 注释规范

## 目的

统一 kernel_master_ws 仓库内所有注释（行注释、块注释、doc comment）的语言、写法与取舍标准。
注释的目标是降低读者理解代码意图的成本，而不是复述代码本身。本规范适用于全部 8 个 crate 的
新代码；存量注释的处理方式见各规则说明与文末"现状与例外"。

## 1. 注释语言：新代码统一简体中文

新写入的注释（包括 `//` 行注释与 `///` doc comment）必须使用简体中文。标识符、类型名、
HTTP 方法、报错串等技术名词保持英文原文，不做翻译。

存量英文注释禁止批量重写；当改动触及某段代码时，顺手将其周边注释翻译为中文。

✅ 新代码（`yell/src/service/notification_router.rs`）：

```rust
// 渲染各渠道 JSON: HashMap<channel_type, rendered_payload>
let rendered = template.render(&variables);
```

✅ 存量英文注释在改动时顺手翻译（示例为 `watchman-backend/src/middleware/auth_middleware.rs`
中的既有注释，翻译后即符合规约）：

```rust
// 白名单
let bypass = GLOBAL_CONFIG
    .read()
    .unwrap()
    .authenticate_bypass
    .iter()
    .any(|p| req.path().starts_with(p));
```

## 2. doc comment（`///`）用于公开 API

公开的模块、结构体、trait、函数必须写 `///` doc comment，内容覆盖契约而非实现：做什么、
参数约束（必填/取值范围/单位）、错误语义（何时返回 `Err`、错误内容的构成）。`pub(crate)`
及以下的内部项可以只在行为不直观时写。

✅ 契约式注释（`yell/src/service/channel.rs`）：

```rust
/// 发送前适配请求内容（默认不修改）
/// 简单推送渠道（Bark/Gotify 等）可覆写此方法，对 HTML 等不适配格式做降级处理
fn prepare_request(&self, request: NotificationRequest) -> NotificationRequest {
    request
}
```

✅ 参数与错误语义（`yell/src/service/notification_router.rs`）：

```rust
/// 发送消息到指定渠道列表
///
/// recipients: Vec<(channel_type, recipient, instance)>
/// 各 channel_type 自行解释 recipient 的含义（邮件地址/用户ID/群组ID/openID/token等）
/// instance 为配置实例名，必填
pub async fn send_to_channels<'a>(...)
```

handler 的路由标注是本仓库的既有惯例（见 `watchman-backend/src/api/account_manage.rs`），
推荐保留，它本身就是"做什么"的有效说明：

```rust
// GET api/auth/me/{id}
pub async fn get_me(...)
```

❌ 只有一行复述签名的 doc comment（`watchman-backend/src/api/subsys_manage.rs` 中的现状，
不推荐模仿）：

```rust
/// POST api/subsystem_call/call_subsystem
pub async fn call_subsys(...)
```

## 3. 行注释解释 why，不复述 what

行注释必须回答"为什么这样写"：业务规则、历史坑、绕过某个限制的原因。代码本身已经能
表达的"做什么"禁止再写一遍。

✅ 解释意图与后果（`watchman-backend/src/middleware/auth_middleware.rs`）：

```rust
// 把 uid 塞进 extensions，下游中间件或 handler 都能拿到
req.extensions_mut().insert(uid);
```

❌ 复述代码的注释（`watchman-backend/src/model/user.rs`、`watchman-backend/src/model/group.rs`
等 model 文件中大量存在的现状，新代码禁止出现）：

```rust
// query implement
pub fn get_user_by_username(...)
```

`// query implement`、`// update implement`、`// used to implement the middleware service`
这类注释删除后信息量为零损失，属于必须避免的类型。

## 4. 禁止注释掉的死代码入库

任何以注释形式保留的代码（常量、函数、配置项、调试日志）禁止提交入库。需要找回时查
git 历史，git 历史即档案。调试用的 `log_debug!` 要么删除，要么保留为正式日志并控制好级别。

❌ 注释掉的常量（`watchman-backend/src/middleware/auth_middleware.rs` 顶部现状，规约是删除——
绕过清单已迁移到配置文件的 `authenticate_bypass` / `permit_bypass`）：

```rust
// const AUTHENTICATE_BYPASS: [&str; 5] = [
//     "/api/auth/signup",
//     "/api/auth/login",
//     "/webhook",
//     "/api/hey",
//     "/api/reload",
// ];
```

❌ 注释掉的调试日志与中间件注册（`watchman-backend/src/middleware/auth_middleware.rs` 中的
`// log_debug!("Parsing token...");`、`watchman-backend/src/main.rs` 中的
`// .wrap(crate::middleware::auth_middleware::Authentication)`，规约同样是删除或启用，
不得长期悬挂）。

## 5. README 与文档双语成对

文档沿用双语成对惯例，两版结构必须完全对应，改动时同步更新：

- crate 级说明：`Readme.md`（英文）+ `Readme_ZH-CN.md`（中文），如 `share-lib/Readme.md`
  与 `share-lib/Readme_ZH-CN.md`。
- `doc/` 规范文档：中文 `-CN.md` 为主稿，英文 `-EN.md` 为忠实翻译，如本篇的
  `doc/comments-CN.md` 与 `doc/comments-EN.md`。

## 6. TODO/FIXME 必须带上下文

`TODO` / `FIXME` 必须说明完成条件或关联 issue，让后来者能判断何时可以动手、做完什么样
算完成。裸的 `// TODO: xxx` 禁止提交。

✅ 带完成条件：

```rust
// TODO: 接入 wecom 渠道——等 WeComChannel 实现 Channel trait 后取消注释并注册（见 yell 渠道规划 issue）
```

❌ 无上下文（`yell/src/service/notification_router.rs` 中的现状，不推荐模仿）：

```rust
// TODO: 注册更多渠道
```

## 现状与例外

- **注释语言中英混用**。老 crate（watchman-backend、cmdb-backend、cloud-api、file-agent、
  share-lib）以英文注释为主；yell 与 jc-commander 的新代码（如 `jc-commander/src/util/scheduler.rs`）
  为中文；`watchman-backend/src/middleware/auth_middleware.rs` 单文件内即中英并存（老
  `Authentication` 段英文、新 `JwtAuth`/`PermissionCheck` 段中文）。按本规范第 1 条渐进收敛，
  不做一次性全量翻译。
- **注释掉的死代码存量较多**。除上文列举的 `auth_middleware.rs` 与 `main.rs` 外，各 crate
  的 `middleware/auth_middleware.rs` 是从 watchman 复制分化的，同样带着注释掉的
  `AUTHENTICATE_BYPASS`/`PERMIT_BYPASS` 与 `log_debug!`。历史原因是早期通过注释开关调试
  与鉴权绕过；现在绕过清单走配置文件、日志走 `log_level`，注释代码已失去用途，改动到
  对应文件时顺手删除。
- **复述式注释存量**。`watchman-backend/src/model/` 下多个文件存在 `// query implement` /
  `// update implement` 式注释，属第 3 条的反面教材，随改动顺手清理。
- **TODO 普遍缺上下文**。`yell/src/service/notification_router.rs` 的两处 TODO 均未关联
  issue 或完成条件，按第 6 条补全。
- **yell 尚无 README**。其余 7 个 crate 与根目录均有 `Readme.md` + `Readme_ZH-CN.md` 成对
  文件，yell 是例外，补齐时按第 5 条成对创建。
