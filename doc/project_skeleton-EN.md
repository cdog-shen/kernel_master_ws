# Project Skeleton Conventions

## Purpose

This document defines the standard directory skeleton and fixed components for every microservice crate in kernel_master_ws, so that new contributors (human or AI) can navigate any crate quickly. `watchman-backend` is the most complete reference implementation; new crates must copy their skeleton from it. This document also records the legitimate per-crate variants and the current gaps between reality and the conventions.

## The Standard Five-Layer Crate Skeleton

The `src/` directory of every HTTP service crate must be organized into the following layers (as in `watchman-backend/src/`):

```
src/
├── main.rs              # startup chain
├── config/              # config layer: mod.rs / app.rs / server.rs
├── api/                 # HTTP handler layer
├── service/             # business logic layer
├── model/               # diesel data layer (incl. schema.rs)
└── middleware/          # auth_middleware.rs (watchman-backend only; subsystem crates use the shared share-lib auth middleware and have no local file)
```

Calls must flow in one direction only: `api` → `service` → `model`. Handlers must not touch diesel directly, and model must not depend back on api/service.

### The main.rs Startup Chain

The startup order of `main()` is fixed: load config → `init_logger` → build the r2d2 pool → start `HttpServer`. See `watchman-backend/src/main.rs`:

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

Routes are mounted uniformly via `.configure(config::app::config_services)` (see `watchman-backend/src/main.rs:95`); routes must not be registered ad hoc inside `main.rs`.

### The Three Files of config/

- `config/mod.rs`: only `pub mod app; pub mod server;`.
- `config/app.rs`: registers all routes in one place; the entry point signature is fixed as `pub fn config_services(cfg: &mut web::ServiceConfig)`. All resources live under `web::scope("/api")`; resources requiring authentication chain `.wrap(JwtAuth).wrap(PermissionCheck)`. See `watchman-backend/src/config/app.rs:8`.
- `config/server.rs`: defines the `AllConfigs` struct, the `reload()` method, and the two once_cell statics `CONFIG_FILE_HANDLE` and `GLOBAL_CONFIG`:

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

Config parsing must go through `share_lib::cfg_reader::read_config`; crates must not implement their own TOML parsing.

### The api/ Handler Layer

One file per resource (e.g. `account_manage.rs`, `group_manage.rs`), exported uniformly via `pub mod` in `api/mod.rs`. Handlers only parse parameters and assemble responses; business logic must live in `service/`.

### The service/ Business Logic Layer

One-to-one with api (`account_manage.rs` → `account_service.rs`). Functions return `Result<MailManOk<...>, MailManErr<...>>`, which the handler converts into an HTTP response. See `login()` in `watchman-backend/src/service/account_service.rs`.

**A pass-through orchestration layer is also a valid form.** The orchestration layer does not always have to contain multi-step orchestration: when a service performs a single model call and then translates the model error code into a `MailManErr` with a uniformly wrapped return value (e.g. the 18 isomorphic pass-through services in `cmdb-backend`, organized per subsystem), it still fulfills the core duty of the orchestration layer — translating model error codes into the MailMan system and wrapping the uniform response — while reserving a hook for future real orchestration logic. Such pass-through services must be kept; they must not be deleted or inlined back into handlers just because they "look hollow".

### The model/ Diesel Data Layer

One file per table, containing a fixed set of three structs plus query/update `impl` blocks, as in `watchman-backend/src/model/user.rs`:

- `XxxModel`: `Queryable, Selectable, Insertable`, fields matching the table columns one-to-one;
- `XxxInputStream`: all-`Option` fields, additionally `AsChangeset`, carrying request input and updates;
- `XxxOutputStream`: all-`Option` fields, containing only externally exposable columns (sensitive fields such as `passwd` are stripped).

Table definitions are centralized in `model/schema.rs` (generated by the diesel CLI); they must not be hand-written.

### Database Migrations (Decision)

Migration files live in each crate's `migrations/` directory, and every migration must provide both `up.sql` and `down.sql`. **Migrations are always executed manually with the diesel CLI** (`diesel migration run`); services never run migrations automatically at startup. Rationale: multiple services share the same PostgreSQL instance (CMDB also hosts tables of other subsystems), so runtime auto-migration carries concurrency and permission risks. The `diesel_migrations` dependency is declared but currently unused; it is kept on purpose — as preparation for embedding migrations in the future, and because removing it changes no behavior.

### Responsibilities of the Cleaning Layer

The cleaning layer means the api handlers and their same-layer helper modules. Two conclusions have been adopted as official conventions:

1. **The `from_map` cleaning functions stay in the model files** and are not moved to the api layer. Rationale: they are naturally cohesive with the three-struct definitions, and moving them would cause repo-wide churn with no payoff. If new code needs complex cleaning logic, a dedicated cleaning module may be created in the api layer.
2. **Whitelist validation of GET filter keys and value-type validation belong to the cleaning layer**: before passing a filter map through, the handler strips unknown keys and wrongly-typed values against a per-table whitelist (stripping is semantically equivalent to the model's silent ignoring, so no new 400 error may be introduced). The filter-key-to-column mapping stays in the model's `get_*_with_filter` as part of the model's atomic responsibility and is not hoisted.

### middleware/ Auth Middleware

Auth middleware falls into two categories by crate role:

- **watchman-backend (the authentication source)**: its local `middleware/auth_middleware.rs` provides the two actix Transforms `JwtAuth` and `PermissionCheck`, applied on demand in `config/app.rs`; their logic shares one implementation with `service/auth_service.rs` (the `/api/auth/verify` origin-auth endpoint), and the two must stay semantically identical.
- **Subsystem crates**: all use the share-lib `UserAuth` middleware (`share-lib/src/middleware/user_auth.rs`); the local `auth_middleware.rs` files have been deleted. See the "Cross-Cutting Components" section below.

## The Non-DB Atomic Operations Layer

Atomic operations invoked by the orchestration layer (`service/`) are owned by type as follows; scattering their implementations inside `service/` is forbidden:

- **DB atomic operations**: belong to each crate's `model/` (see the previous section) — one file per table, carrying diesel queries/updates.
- **Non-DB atomic operations** (outbound HTTP, message queues, process spawning, etc.): consolidated in the `infrastructure` module of `share-lib` (`share-lib/src/infrastructure/`); crates opt in per feature:
  - `share_lib::infrastructure::http_client` — synchronous HTTP atomic operations based on ureq (`get` / `delete` / `post_json` / `put_json`), behind feature `http`;
  - `share_lib::infrastructure::mq_client` — asynchronous MQ atomic operations based on lapin (`connect` / `declare_quorum_queue` / `publish_json`), behind feature `mq`; the consumer-side consume loop is business-specific and stays in each crate;
  - `share_lib::infrastructure::process_runner` — synchronous process execution based on std::process (`run` returns `ProcessOutput { stdout, stderr, exit_code }`), not feature-gated and always available.
- **Crate-specific atomic operations**: if an atomic operation is used by only one crate with no prospect of reuse, it may stay in that crate's `util/` (e.g. the filesystem primitives in `file-agent/src/util/file_op.rs`).

All atomic operations report errors through the MailMan system (`MailManErr::new`, 500-series codes, level 0/1 per semantics), to be propagated or converted by the orchestration layer.

## Cross-Cutting Components (middleware / scheduler)

Components such as middleware and schedulers cut across the `api` → `service` → `model` layers without belonging to any of them, but they must obey the same responsibility and IO-consolidation constraints:

- **Auth middleware**: a cross-cutting layer outside the three layers; it must not carry business orchestration. Subsystem auth middleware has been consolidated into share-lib — the single source is `share-lib/src/middleware/user_auth.rs` (`UserAuth`/`UserAuthConfig`, features `web` + `http`):
  - **Config source**: share-lib does not hold any crate's `GLOBAL_CONFIG`; the caller injects a `UserAuthConfig` when constructing the middleware in `main.rs` — `master_addr`/`master_port` (origin-auth address, reusing the same config keys as refresh_master; the fields are compiled out in individual mode), the local `subsys_uuid`, and the `authenticate_bypass` whitelist (any path-prefix match passes, as do OPTIONS preflights);
  - **Bearer/uuid dual branch** (by `Authorization` header scheme, case-insensitive): `Bearer <jwt>` is the new direct-connection path — origin authentication via `POST /api/auth/verify` on watchman (carrying the local uuid as caller credential); upstream 401/403 are mapped through as-is, network failures/timeouts and any other non-2xx become 503, and on success the `uid` (i32) is written into the request extensions before passing through. `uuid <subsys_uuid>` is the legacy watchman-forwarded path — it is compared against the local config and passed (kept for the transition; delete this branch when the legacy link is retired);
  - **`individual` conditional compilation**: with feature `individual` enabled, the origin-auth (Bearer) branch is compiled out and only the uuid-comparison branch remains, so a subsystem can run standalone without watchman; without `individual` and without feature `http`, compilation is rejected outright via `compile_error!`;
  - watchman-backend is the authentication source and keeps its local `JwtAuth`/`PermissionCheck` (which may call model directly for JWT parsing and permission queries); it does not adopt this middleware.
- **Scheduler**: `jc-commander`'s `util/scheduler.rs` (a timing-wheel scheduler) is positioned as a standalone scheduler component that belongs to none of the three layers. Its constraints match the orchestration layer's: DB calls must go through `model/`, and MQ calls must go through the `share_lib::infrastructure::mq_client` atomic module — inlining diesel or lapin code inside the scheduler is forbidden.
- **Other crate-specific cross-cutting components** (e.g. file-agent's SegQueue in-memory queue) follow the same principle: all IO goes through the atomic layer (model or share-lib infrastructure) and must not be inlined inside the component.

## Fixed Components

Every HTTP service crate must provide the following two endpoints:

1. **Health check** `api/hey_hi_hello.rs`: `GET /api/hey` returns `"hi hello!"`.

   ```rust
   // watchman-backend/src/api/hey_hi_hello.rs
   pub async fn hey() -> HttpResponse {
       HttpResponse::Ok().body("hi hello!".to_string())
   }
   ```

2. **Hot reload** `api/system_manage.rs`: `POST /api/reload` calls `GLOBAL_CONFIG.write().unwrap().reload()`; errors are converted into responses via `MailManErrResponser::mapping_from_mme`. See `watchman-backend/src/api/system_manage.rs`.

## Variants

The skeleton permits the following registered variants; no other layers may be added or removed:

- **jc-worker**: a pure MQ consumer with no `api/`, `middleware/`, or `model/`. Its config layer is `config/worker.rs` (`jc-worker/src/config/worker.rs`); the startup chain switches to `#[tokio::main]` plus lapin connect/queue-declare/consume (`jc-worker/src/main.rs`); business logic lives in `service/task.rs` and `service/json_rpc.rs`.
- **file-agent**: no database — `mod model;` is commented out as a whole in `main.rs` and there is no r2d2 pool; multipart upload logic is in `service/file_manage.rs`, and filesystem primitives are consolidated in `util/file_op.rs`.
- **yell**: the business layer uses the plural directory `services/`, subdivided per notification channel (`bark/`, `gotify/`, `mail/`, `teams/`, `teams_hook/`, `webhook/`); `services/channel.rs` defines the `Channel` trait that every channel must implement (`channel_type` / `preflight` / `dispatch_template`), and `NotificationRouter` in `services/notification_router.rs` registers and dispatches all channels.
- **cmdb-backend**: `model/` and `service/` add one more level of subdirectories per subsystem (`km/`, `cloudserver/`, `yell/`, etc.), e.g. `cmdb-backend/src/model/km/cloud_account.rs`.
- **jc-commander**: additionally has `util/scheduler.rs` (the scheduler), which is a crate-private component and not a skeleton variant.

## New Crate Onboarding Checklist

When adding a crate, every item below must be completed — none may be skipped:

- [ ] Add the new crate to `members` in the root `Cargo.toml`;
- [ ] In the new crate's `Cargo.toml`, inherit all dependencies via `xxx.workspace = true`; version numbers are forbidden (see `watchman-backend/Cargo.toml`);
- [ ] Add the new crate to `DIRS`/`BINS` in `build/build_all_ws.sh`, `build/check_all_ws.sh`, `build/clippy_all_ws.sh`, and the other build scripts;
- [ ] Register the service in `docker-compose.yaml`, assign a port (existing range: 8000 watchman, 9001–9005 for cmdb / cloud-api / jc-commander / file-agent / yell respectively), and configure a healthcheck;
- [ ] Provide a `<name>.template.toml` config template;
- [ ] Provide bilingual docs `README.md` and `Readme_ZH-CN.md`;
- [ ] Copy the five-layer skeleton from section 2 (trim per the section 4 variants for crates without a database or HTTP), and implement the `/api/hey` and `/api/reload` fixed components.

## Current Status and Exceptions

- The `/api/reload` endpoint is currently registered only by `watchman-backend` in `config/app.rs` (`watchman-backend/src/config/app.rs:20`). The other crates implement `AllConfigs::reload()` and use it for startup loading, but expose no HTTP hot-reload endpoint; `file-agent`'s and `yell`'s `api/system_manage.rs` host `refresh_master` (re-registering with watchman), not reload.
- `/api/hey` is registered for both GET and POST in watchman-backend, but only for POST in the other crates (e.g. `file-agent/src/config/app.rs:11`); callers should use POST for compatibility.
- `yell` was historically left out of the build scripts and docker-compose for a long time. The build scripts now include `yell`, `docker-compose.yaml` registers the `yell` service (`docker-compose.yaml:134`), and the bilingual `README.md` / `Readme_ZH-CN.md` are in place — this historical omission is closed.
- The local `middleware/auth_middleware.rs` files of all subsystem crates have been deleted in favor of the shared share-lib auth middleware (`share-lib/src/middleware/user_auth.rs`); only `config/server.rs` still diverges per crate (it depends on each crate's own model/config), which is intentional non-shared code and is not moved into share-lib.
- The `refresh_master` route is inconsistent across crates: `/api/manage/refresh_master` in cmdb-backend / yell / file-agent, but `/api/refresh_master` in cloud-api / jc-commander; each crate's `authenticate_bypass` whitelist matches its actual route. This route is cfg-gated out in `individual` mode.
