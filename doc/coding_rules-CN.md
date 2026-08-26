# 编码规范

## 目的

本文档规定 kernel_master_ws 仓库的编码规则，面向项目开发者与 AI 编码助手。
目标是让 8 个 crate 在格式化、依赖管理、错误处理上保持一致，降低 review 与维护成本。
规则分级：「必须 / 禁止」是硬约束，「推荐 / 可以」是建议。

## 格式化与 Lint：rustfmt / clippy 是硬门槛

提交前必须通过格式化检查与 clippy，二者均按 workspace 整体执行：

```sh
build/fmt_all_ws.sh -- --check    # 等价于 cargo fmt --all -- --check
build/clippy_all_ws.sh            # 等价于 cargo clippy --workspace --all-targets
```

- 格式化配置以根目录 `rustfmt.toml` 为唯一来源，当前为 `edition = "2024"`、`style_edition = "2024"`。
  禁止在编辑器里套用个人格式化规则覆盖它。
- 禁止用 `#[allow(...)]` 或 `#[rustfmt::skip]` 绕过 lint / 格式化，除非有明确注释说明原因。
- clippy 当前未开启 `-D warnings`（仅记录基线警告，见「现状与例外」），但规则上要求：
  新代码不得引入任何新的 clippy 警告。改动文件前建议先跑一次 clippy 记录基线，
  改完对比确认无新增。

## 依赖管理：版本号只允许出现在根 Cargo.toml

workspace 化改造后，所有第三方依赖的版本统一声明在根 `Cargo.toml` 的
`[workspace.dependencies]` 中，成员 crate 一律用 `xxx.workspace = true` 继承，
成员 crate 的 `Cargo.toml` 中禁止出现任何版本号。

✅ 正确写法（`watchman-backend/Cargo.toml`）：

```toml
[dependencies]
once_cell.workspace = true
actix-web.workspace = true
serde.workspace = true
share-lib = { workspace = true, features = ["web"] }
```

❌ 错误写法（成员 crate 中出现版本号）：

```toml
[dependencies]
serde = "1.0.228"              # 禁止：版本号必须上收根 Cargo.toml
share-lib = { path = "../share-lib" }  # 禁止：share-lib 统一走 workspace 继承
```

- 新增第三方依赖前，必须先在根 `Cargo.toml` 的 `[workspace.dependencies]`
  中确认没有等价物（例如 JSON 处理统一用 `serde_json`，HTTP 客户端统一用 `ureq`），
  禁止平行引入功能重复的 crate。
- 确实需要新增时，只在根 `Cargo.toml` 添加版本声明，成员 crate 侧只写
  `xxx.workspace = true`。
- `share-lib` 的路径声明只允许出现在根 `Cargo.toml`（`share-lib = { path = "./share-lib" }`），
  成员 crate 一律通过 `workspace = true` 引用。

## panic / unwrap / expect 边界

### 启动期允许快速失败

`main.rs` 启动阶段、配置加载（`GLOBAL_CONFIG::reload`）这类「失败即无法运行」
的路径，允许使用 `expect` / `panic!` 快速失败，让进程尽早暴露问题。

✅ 允许（`watchman-backend/src/main.rs`）：

```rust
match server::GLOBAL_CONFIG.write().unwrap().reload() {
    Ok(_) => {
        MailManOk::new(200, "config load DONE", None::<&str>);
    }
    Err(e) => {
        panic!("config load error! {e:?}");
    }
}
// ...
let pool = diesel::r2d2::Pool::builder()
    .build(manager)
    .expect("Failed to create pool.");
```

### 请求处理路径禁止裸 unwrap

`api/`、`service/`、`model/` 等请求处理路径上，禁止用裸 `unwrap()` /
`expect()` 处理可恢复错误——一次 panic 会打断整个 worker 的请求处理。
错误必须转换为 `MailManErr`，经 `MailManErrResponser` 映射为 HTTP 响应
（详见 `doc/error_handling-CN.md`）。

✅ 正确写法（`watchman-backend/src/service/account_service.rs`）：

```rust
pub fn new_user<'a>(
    user_to_creat: UserInputStream,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<String>> {
    match UserModel::new_user(&user_to_creat, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service: Create user", Some(msg))),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(400, "Service: Create user", Some(msg.1), 1)),
            _ => Err(MailManErr::new(500, "Service: Create user", Some(msg.1), 1)),
        },
    }
}
```

❌ 禁止（请求路径上的裸 unwrap，`watchman-backend/src/middleware/auth_middleware.rs:381`）：

```rust
let user = UserModel::get_user_by_username(&username, &mut conn).unwrap();
```

注：`GLOBAL_CONFIG.read().unwrap()` 这类 `RwLock` 加锁的 unwrap 是另一回事——
锁中毒本身就是不可恢复状态，属可接受用法，不纳入本条限制。

## 编排层函数统一为 async

编排层（`service/` 目录，以及承担编排职责的函数，如 yell 的 `NotificationRouter`
方法）的执行函数**必须**声明为 `async fn`，即使当前实现完全是同步的。

- 理由一（兼容性）：编排层未来必然调用 async 原子操作（MQ 投递、异步 HTTP 等），
  提前统一 async 可避免届时签名变更级联破坏全部调用方。
- 理由二（风格一致性）：全 workspace 的编排接口形态统一，handler 侧一律
  `service::xxx(...).await`，消除 sync/async 混用的判断成本。

✅ 正确写法（在既有同步签名基础上加 `async`）：

```rust
pub async fn new_user<'a>(
    user_to_creat: UserInputStream,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<String>> {
    // ...
}
```

约束与说明：

- 本条只约束编排层的**签名形态**，不改变底层行为：model 层的 diesel 同步操作
  保持现状，async 化不等于非阻塞；如需真正不阻塞 worker 线程，另行评估
  `web::block`，不属于本条范围。
- model 层（原子操作）不强制 async，跟随其所用客户端的同步/异步形态。
- 存量同步 service 函数属待迁移现状（迁移计划见根目录 `todo.md`）：
  新增编排函数必须 async；改动到既有同步函数时顺手迁移，迁移必须同步更新
  全部调用方（handler 改为 `.await` 调用）。

## 禁止注释掉的死代码入库

禁止把注释掉的旧实现、调试片段提交入库。历史代码用 git 找回，不用注释保存。
细节见注释规范 `doc/comments-CN.md`。

## 最小改动原则

- 只做任务要求的改动：不顺手重构、不清理与本次任务无关的代码、不改无关文件的格式。
- 不引入投机性抽象：没有第二处真实调用点之前，不抽公共函数 / 泛型 / 配置项。
  三处相似代码再考虑收敛，且公共代码优先收敛进 `share-lib`，禁止跨 crate 复制粘贴。
- 接口变更时，必须同步更新所有调用方；但除接口适配外，不改动调用方的既有逻辑。

## 现状与例外

以下为当前代码与上述规则的实际差距，属待收敛的现状，不是新的写法范例：

- **clippy 未开 `-D warnings`**：`build/clippy_all_ws.sh` 执行的是
  `cargo clippy --workspace --all-targets`，警告目前仅记录、不阻断。
  规则上仍要求新代码零新增警告；待基线清零后可考虑在 CI 加 `-D warnings`。
- **middleware / service 中的 `pool.get().unwrap()`**：连接池取连接失败在请求路径上
  直接 panic 不符合第 3 节规则，但目前遍布各处，如
  `watchman-backend/src/middleware/auth_middleware.rs`（`pool.get().unwrap()` 出现于
  约 140、146、151、157、167、365、486 行）与
  `watchman-backend/src/service/account_service.rs`、`access_service.rs` 等。
  属历史遗留，待逐步收敛为 `MailManErr` 错误返回；新代码禁止效仿。
- **`config/server.rs` 的逐字段 `expect`**：如
  `watchman-backend/src/config/server.rs` 中
  `.expect("Config path server_config:log_path (string) not found")` 等。
  因 `reload()` 同时服务启动期与 `/api/reload` 热重载（请求路径），严格说不完全符合
  第 3 节的边界划分，属待收敛项；现阶段维持现状。
- **存量注释掉的代码**：`watchman-backend/src/main.rs`、
  `watchman-backend/src/config/server.rs` 中仍有注释掉的字段与逻辑
  （如 `pub_key_path` / `pri_key_path` 相关行），为密钥方案调整期的遗留，待清理。
