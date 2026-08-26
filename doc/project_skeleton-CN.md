# 项目骨架规范

## 目的

本规范定义 kernel_master_ws 中各微服务 crate 的标准目录骨架与固定组件，保证新成员（人或 AI）在任意 crate 中都能快速定位代码。`watchman-backend` 是骨架最完整的参考实现，新 crate 必须以它为模板复制骨架。本文同时记录各 crate 的合法变体与当前现状同规约的差距。

## 标准 crate 五层骨架

每个 HTTP 服务 crate 的 `src/` 必须按以下层次组织（以 `watchman-backend/src/` 为准）：

```
src/
├── main.rs              # 启动链
├── config/              # 配置层：mod.rs / app.rs / server.rs
├── api/                 # HTTP handler 层
├── service/             # 业务逻辑层
├── model/               # diesel 数据层（含 schema.rs）
└── middleware/          # auth_middleware.rs
```

各层调用方向必须是单向的：`api` → `service` → `model`。禁止 handler 直接访问 diesel，禁止 model 反向依赖 api/service。

### main.rs 启动链

`main()` 的启动顺序是固定的，必须依次执行：加载配置 → `init_logger` → 建 r2d2 连接池 → 启动 `HttpServer`。参考 `watchman-backend/src/main.rs`：

```rust
// reload config
match server::GLOBAL_CONFIG.write().unwrap().reload() {
    Ok(_) => {
        MailManOk::new(200, "config load DONE", None::<&str>);
    }
    Err(e) => {
        panic!("config load error! {e:?}");
    }
}

// init logger
logger::init_logger(
    &server::GLOBAL_CONFIG.read().unwrap().log_path,
    &server::GLOBAL_CONFIG.read().unwrap().log_level,
);

// init postgres connection pool
let manager =
    ConnectionManager::<PgConnection>::new(&server::GLOBAL_CONFIG.read().unwrap().db_str);
let pool = diesel::r2d2::Pool::builder()
    .build(manager)
    .expect("Failed to create pool.");
```

路由挂载统一走 `.configure(config::app::config_services)`（见 `watchman-backend/src/main.rs:95`），禁止在 `main.rs` 里直接散落注册路由。

### config/ 三个文件

- `config/mod.rs`：只做 `pub mod app; pub mod server;`。
- `config/app.rs`：集中注册全部路由，入口函数签名固定为 `pub fn config_services(cfg: &mut web::ServiceConfig)`。所有资源挂在 `web::scope("/api")` 下，需要鉴权的资源链式 `.wrap(JwtAuth).wrap(PermissionCheck)`。参考 `watchman-backend/src/config/app.rs:8`。
- `config/server.rs`：定义 `AllConfigs` 结构体、`reload()` 方法、`CONFIG_FILE_HANDLE` 与 `GLOBAL_CONFIG` 两个 once_cell 静态单例：

```rust
// watchman-backend/src/config/server.rs
pub static CONFIG_FILE_HANDLE: Lazy<Mutex<File>> = Lazy::new(|| {
    let path = std::env::current_dir()
        .expect("Unable to get workspace path")
        .join("watchman.toml");
    let file = File::open(&path).expect("Unable to open config file");
    Mutex::new(file)
});

pub static GLOBAL_CONFIG: Lazy<RwLock<AllConfigs>> = Lazy::new(|| RwLock::new(AllConfigs::new()));
```

配置解析必须走 `share_lib::cfg_reader::read_config`，禁止各 crate 自实现 TOML 解析。

### api/ handler 层

每个资源一个文件（如 `account_manage.rs`、`group_manage.rs`），`api/mod.rs` 统一 `pub mod` 导出。handler 只做参数解析与响应组装，业务逻辑必须下沉到 `service/`。

### service/ 业务逻辑层

与 api 一一对应（`account_manage.rs` → `account_service.rs`）。函数返回 `Result<MailManOk<...>, MailManErr<...>>`，由 handler 侧转成 HTTP 响应。参考 `watchman-backend/src/service/account_service.rs` 的 `login()`。

### model/ diesel 数据层

每张表对应一个文件，内部固定三个结构体 + 查询/更新 `impl` 块，参考 `watchman-backend/src/model/user.rs`：

- `XxxModel`：`Queryable, Selectable, Insertable`，字段与表列一一对应；
- `XxxInputStream`：全 `Option` 字段，附加 `AsChangeset`，承接请求输入与更新；
- `XxxOutputStream`：全 `Option` 字段，只含可对外输出的列（剔除 `passwd` 等敏感字段）。

表定义集中在 `model/schema.rs`（diesel CLI 生成），禁止手写。

### middleware/auth_middleware.rs

提供 `JwtAuth` 与 `PermissionCheck` 两个 actix Transform，在 `config/app.rs` 中按需 wrap。该文件各 crate 因依赖自身 model 已分化，修改时必须逐 crate 同步评估。

## 非 DB 原子操作层

编排层（`service/`）调用的原子操作按类型归属如下，禁止在 `service/` 内直接散落实现：

- **DB 原子操作**：归各 crate 的 `model/`（见上一节），每张表一个文件，承接 diesel 查询/更新。
- **非 DB 原子操作**（HTTP 外呼、消息队列、进程派生等）：统一收敛在 `share-lib` 的 `infrastructure` 模块（`share-lib/src/infrastructure/`），各 crate 按需通过 feature 引入：
  - `share_lib::infrastructure::http_client` — 基于 ureq 的同步 HTTP 原子操作（`get` / `delete` / `post_json` / `put_json`），feature `http`；
  - `share_lib::infrastructure::mq_client` — 基于 lapin 的异步 MQ 原子操作（`connect` / `declare_quorum_queue` / `publish_json`），feature `mq`；消费端的消费循环与业务强相关，继续留在各 crate；
  - `share_lib::infrastructure::process_runner` — 基于 std::process 的同步进程执行（`run` 返回 `ProcessOutput { stdout, stderr, exit_code }`），无 feature 门控，常驻可用。
- **crate 特有的原子操作**：若某原子操作只有一个 crate 使用且与其他 crate 无复用前景，可留在本 crate 的 `util/`（如 `file-agent/src/util/file_op.rs` 的文件系统原语）。

所有原子操作的错误一律走 MailMan 体系（`MailManErr::new`，500 系 code，level 按语义取 0/1），由编排层继续上抛或转换。

## 固定件

以下两个端点每个 HTTP 服务 crate 都必须具备：

1. **健康检查** `api/hey_hi_hello.rs`：`GET /api/hey` 返回 `"hi hello!"`。

   ```rust
   // watchman-backend/src/api/hey_hi_hello.rs
   pub async fn hey() -> HttpResponse {
       HttpResponse::Ok().body("hi hello!".to_string())
   }
   ```

2. **热重载** `api/system_manage.rs`：`POST /api/reload` 调用 `GLOBAL_CONFIG.write().unwrap().reload()`，错误经 `MailManErrResponser::mapping_from_mme` 转响应。参考 `watchman-backend/src/api/system_manage.rs`。

## 变体

骨架允许以下已登记的变体，除此之外不得自行增删层次：

- **jc-worker**：纯 MQ 消费者，无 `api/`、`middleware/`、`model/`。配置层为 `config/worker.rs`（`jc-worker/src/config/worker.rs`），启动链改为 `#[tokio::main]` + lapin 连接/建队/消费（`jc-worker/src/main.rs`），业务逻辑在 `service/task.rs`、`service/json_rpc.rs`。
- **file-agent**：无数据库，`main.rs` 中 `mod model;` 整体注释掉，无 r2d2 池；multipart 上传逻辑在 `service/file_manage.rs`，文件系统原语收敛在 `util/file_op.rs`。
- **yell**：业务层用复数目录 `services/`，按通知渠道（`bark/`、`gotify/`、`mail/`、`teams/`、`webhook/`、`wecom/`）分目录；`services/channel.rs` 定义所有渠道必须实现的 `Channel` trait（`channel_type` / `build_message` / `prepare_request` / `send` / `send_template`），`services/notification_router.rs` 的 `NotificationRouter` 统一注册与分发渠道。
- **cmdb-backend**：`model/` 与 `service/` 先按子系统（`km/`、`cloudserver/`、`yell/` 等）再分一层目录，如 `cmdb-backend/src/model/km/cloud_account.rs`。
- **jc-commander**：额外有 `util/scheduler.rs`（调度器），属 crate 私有组件，不算骨架变体。

## 新 crate 接入清单

新增 crate 时必须逐项完成登记，缺一不可：

- [ ] 根 `Cargo.toml` 的 `members` 加入新 crate；
- [ ] 新 crate 的 `Cargo.toml` 依赖全部用 `xxx.workspace = true` 继承，禁止写版本号（参考 `watchman-backend/Cargo.toml`）；
- [ ] `build/build_all_ws.sh`、`build/check_all_ws.sh`、`build/clippy_all_ws.sh` 等脚本的 `DIRS`/`BINS` 加入新 crate；
- [ ] `docker-compose.yaml` 注册服务、分配端口（现有段位：8000 watchman，9001–9005 依次 cmdb / cloud-api / jc-commander / file-agent / yell）并配 healthcheck；
- [ ] 提供 `<name>.template.toml` 配置模板；
- [ ] 提供 `README.md` 与 `Readme_ZH-CN.md` 双语说明；
- [ ] 按本文第二节复制五层骨架（无数据库/无 HTTP 的按第四节变体裁剪），并实现 `/api/hey` 与 `/api/reload` 固定件。

## 现状与例外

- `/api/reload` 端点目前只有 `watchman-backend` 在 `config/app.rs` 中注册（`watchman-backend/src/config/app.rs:20`）。其余 crate 的 `AllConfigs::reload()` 已实现并用于启动加载，但未暴露 HTTP 热重载端点；`file-agent` 的 `api/system_manage.rs` 承载的是 `refresh_master`（向 watchman 刷新注册），不是 reload。
- `/api/hey` 在 watchman-backend 同时注册 GET 与 POST，其余 crate 只注册了 POST（如 `file-agent/src/config/app.rs:11`），调用方应使用 POST 以保证兼容。
- `yell` 历史上曾长期漏登记进 build 脚本与 docker-compose。当前 build 脚本已包含 `yell`，但 `docker-compose.yaml` 中仍无 `yell` 服务；`yell` 也缺少 `README.md` / `Readme_ZH-CN.md`。接入检查清单即为此类遗漏设立，补齐前部署 yell 需手动处理。
- `middleware/auth_middleware.rs` 与 `config/server.rs` 各 crate 已按自身 model/配置分化，属有意的非公共代码，不纳入 share-lib。
