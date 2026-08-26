# Commenting Guidelines

## Purpose

This document standardizes the language, style, and inclusion criteria for all comments
(line comments, block comments, and doc comments) across the kernel_master_ws workspace.
The goal of a comment is to lower the cost of understanding code intent, not to restate
the code itself. These rules apply to new code in all 8 crates; handling of existing
comments is described per rule and in "Current State and Exceptions" at the end.

## 1. Comment Language: Simplified Chinese for New Code

All newly written comments (both `//` line comments and `///` doc comments) must use
Simplified Chinese. Technical terms such as identifiers, type names, HTTP methods, and
error strings stay in their original English form and are not translated.

Existing English comments must not be rewritten in bulk; when a change touches a piece of
code, translate its surrounding comments into Chinese as part of that change.

✅ New code (`yell/src/services/notification_router.rs`):

```rust
// 渲染各渠道 JSON: HashMap<channel_type, rendered_payload>
let rendered = template.render(&variables);
```

✅ Existing English comments are translated when touched (the example below is an existing
comment in `watchman-backend/src/middleware/auth_middleware.rs`; once translated it
complies with the rule):

```rust
// 白名单
let bypass = GLOBAL_CONFIG
    .read()
    .unwrap()
    .authenticate_bypass
    .iter()
    .any(|p| req.path().starts_with(p));
```

## 2. Doc Comments (`///`) for Public APIs

Public modules, structs, traits, and functions must carry `///` doc comments describing
the contract rather than the implementation: what it does, parameter constraints
(required / value ranges / units), and error semantics (when it returns `Err`, what the
error contains). Internal items (`pub(crate)` and below) may be documented only when
their behavior is not self-evident.

✅ Contract-style comment (`yell/src/services/channel.rs`):

```rust
/// 发送前适配请求内容（默认不修改）
/// 简单推送渠道（Bark/Gotify 等）可覆写此方法，对 HTML 等不适配格式做降级处理
fn prepare_request(&self, request: NotificationRequest) -> NotificationRequest {
    request
}
```

✅ Parameter and error semantics (`yell/src/services/notification_router.rs`):

```rust
/// 发送消息到指定渠道列表
///
/// recipients: Vec<(channel_type, recipient, instance)>
/// 各 channel_type 自行解释 recipient 的含义（邮件地址/用户ID/群组ID/openID/token等）
/// instance 为配置实例名，必填
pub async fn send_to_channels<'a>(...)
```

Route annotations on handlers are an established convention in this repository (see
`watchman-backend/src/api/account_manage.rs`) and are recommended — they are themselves
an effective statement of "what":

```rust
// GET api/auth/me/{id}
pub async fn get_me(...)
```

❌ A doc comment that merely restates the route and signature (current state in
`watchman-backend/src/api/subsys_manage.rs`; do not imitate):

```rust
/// POST api/subsystem_call/call_subsystem
pub async fn call_subsys(...)
```

## 3. Line Comments Explain Why, Not What

A line comment must answer "why is it written this way": a business rule, a historical
pitfall, the reason for working around a limitation. Restating what the code already
expresses is forbidden.

✅ Explaining intent and effect (`watchman-backend/src/middleware/auth_middleware.rs`):

```rust
// 把 uid 塞进 extensions，下游中间件或 handler 都能拿到
req.extensions_mut().insert(uid);
```

❌ Comments that restate the code (current state, found in large numbers in
`watchman-backend/src/model/user.rs`, `watchman-backend/src/model/group.rs`, and other
model files; forbidden in new code):

```rust
// query implement
pub fn get_user_by_username(...)
```

Comments like `// query implement`, `// update implement`, and
`// used to implement the middleware service` lose zero information when deleted and are
exactly the kind that must be avoided.

## 4. Commented-Out Dead Code Must Not Be Committed

Keeping code around in commented form (constants, functions, configuration entries,
debug logging) is forbidden. Use git history to recover anything — git history is the
archive. A debug `log_debug!` must either be removed or kept as a real log statement
with a properly controlled level.

❌ Commented-out constants (current state at the top of
`watchman-backend/src/middleware/auth_middleware.rs`; the rule is to delete — the bypass
list has moved to the configuration file as `authenticate_bypass` / `permit_bypass`):

```rust
// const AUTHENTICATE_BYPASS: [&str; 5] = [
//     "/api/auth/signup",
//     "/api/auth/login",
//     "/webhook",
//     "/api/hey",
//     "/api/reload",
// ];
```

❌ Commented-out debug logging and middleware registration (`// log_debug!("Parsing
token...");` in `watchman-backend/src/middleware/auth_middleware.rs`, and
`// .wrap(crate::middleware::auth_middleware::Authentication)` in
`watchman-backend/src/main.rs`; the rule is likewise to delete or enable them — they
must not dangle indefinitely).

## 5. Bilingual Pairs for README and Docs

Documents follow a bilingual pairing convention; the two versions must match in
structure and be updated together:

- Crate-level docs: `Readme.md` (English) + `Readme_ZH-CN.md` (Chinese), e.g.
  `share-lib/Readme.md` and `share-lib/Readme_ZH-CN.md`.
- `doc/` guideline documents: the Chinese `-CN.md` is the master, the English `-EN.md`
  is a faithful translation — e.g. this document's `doc/comments-CN.md` and
  `doc/comments-EN.md`.

## 6. TODO/FIXME Must Carry Context

Every `TODO` / `FIXME` must state its completion condition or link an issue, so a later
reader can tell when action is possible and what "done" looks like. Bare `// TODO: xxx`
comments must not be committed.

✅ With a completion condition:

```rust
// TODO: 接入 wecom 渠道——等 WeComChannel 实现 Channel trait 后取消注释并注册（见 yell 渠道规划 issue）
```

❌ No context (current state in `yell/src/services/notification_router.rs`; do not
imitate):

```rust
// TODO: 注册更多渠道
```

## Current State and Exceptions

- **Comments mix Chinese and English.** Older crates (watchman-backend, cmdb-backend,
  cloud-api, file-agent, share-lib) use mostly English comments; new code in yell and
  jc-commander (e.g. `jc-commander/src/util/scheduler.rs`) uses Chinese;
  `watchman-backend/src/middleware/auth_middleware.rs` mixes both within one file (the
  old `Authentication` section in English, the newer `JwtAuth`/`PermissionCheck`
  sections in Chinese). Converge gradually per rule 1; do not do a one-off wholesale
  translation.
- **A large stock of commented-out dead code.** Beyond the `auth_middleware.rs` and
  `main.rs` examples above, each crate's `middleware/auth_middleware.rs` was copied and
  diverged from watchman's and carries the same commented-out
  `AUTHENTICATE_BYPASS`/`PERMIT_BYPASS` and `log_debug!` lines. The historical reason is
  that debugging and auth bypass were toggled by commenting in the early days; now the
  bypass list lives in the configuration file and logging is governed by `log_level`, so
  the commented code no longer serves a purpose. Delete it when touching the
  corresponding file.
- **Stock of restating comments.** Several files under `watchman-backend/src/model/`
  contain `// query implement` / `// update implement` style comments; they are the
  counterexample for rule 3 and are cleaned up opportunistically as code is touched.
- **TODOs generally lack context.** Neither TODO in
  `yell/src/services/notification_router.rs` links an issue or states a completion
  condition; complete them per rule 6.
- **yell has no README yet.** The other 7 crates and the repository root all have paired
  `Readme.md` + `Readme_ZH-CN.md` files; yell is the exception. When it is added,
  create the pair per rule 5.
