# Coding Rules

## Purpose

This document defines the coding rules for the kernel_master_ws repository,
for both project developers and AI coding assistants.
Its goal is to keep the 8 crates consistent in formatting, dependency management,
and error handling, reducing review and maintenance cost.
Rule levels: "must / must not" are hard constraints; "recommended / may" are advisory.

## Formatting & Lint: rustfmt / clippy Are Hard Gates

Formatting checks and clippy must pass before every commit; both run at workspace level:

```sh
build/fmt_all_ws.sh -- --check    # equivalent to cargo fmt --all -- --check
build/clippy_all_ws.sh            # equivalent to cargo clippy --workspace --all-targets
```

- The root `rustfmt.toml` is the single source of formatting configuration,
  currently `edition = "2024"` and `style_edition = "2024"`.
  Do not override it with personal editor formatting settings.
- Do not use `#[allow(...)]` or `#[rustfmt::skip]` to bypass lints or formatting
  unless a comment clearly explains why.
- clippy currently does not use `-D warnings` (baseline warnings are only recorded;
  see "Current Status & Exceptions"), but the rule is: new code must not introduce
  any new clippy warnings. Before changing a file, run clippy to record the
  baseline; after the change, verify nothing new appeared.

## Dependency Management: Version Numbers Only in the Root Cargo.toml

After the workspace migration, all third-party dependency versions are declared
in `[workspace.dependencies]` of the root `Cargo.toml`. Member crates must inherit
them with `xxx.workspace = true`; no version number may appear in any member
crate's `Cargo.toml`.

✅ Correct (`watchman-backend/Cargo.toml`):

```toml
[dependencies]
once_cell.workspace = true
actix-web.workspace = true
serde.workspace = true
share-lib = { workspace = true, features = ["web"] }
```

❌ Wrong (a version number in a member crate):

```toml
[dependencies]
serde = "1.0.228"              # forbidden: versions belong in the root Cargo.toml
share-lib = { path = "../share-lib" }  # forbidden: share-lib goes through workspace inheritance
```

- Before adding a third-party dependency, you must first check
  `[workspace.dependencies]` in the root `Cargo.toml` for an equivalent
  (e.g. `serde_json` for JSON, `ureq` as the HTTP client). Adding a parallel crate
  with duplicate functionality is forbidden.
- When a new dependency is genuinely needed, declare its version only in the root
  `Cargo.toml`; member crates only write `xxx.workspace = true`.
- The `share-lib` path declaration may appear only in the root `Cargo.toml`
  (`share-lib = { path = "./share-lib" }`); member crates always reference it via
  `workspace = true`.

## Boundaries for panic / unwrap / expect

### Fast Failure Is Allowed at Startup

On paths where failure means the process cannot run at all — the startup phase of
`main.rs` and configuration loading (`GLOBAL_CONFIG::reload`) — `expect` /
`panic!` are allowed so problems surface as early as possible.

✅ Allowed (`watchman-backend/src/main.rs`):

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

### No Bare unwrap on Request-Handling Paths

On request-handling paths (`api/`, `service/`, `model/`), bare `unwrap()` /
`expect()` on recoverable errors is forbidden — a single panic aborts the whole
worker's request processing. Errors must be converted into `MailManErr` and mapped
to HTTP responses via `MailManErrResponser` (see `doc/error_handling-EN.md`).

✅ Correct (`watchman-backend/src/service/account_service.rs`):

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

❌ Forbidden (bare unwrap on a request path, `watchman-backend/src/middleware/auth_middleware.rs:381`):

```rust
let user = UserModel::get_user_by_username(&username, &mut conn).unwrap();
```

Note: lock-acquisition unwraps like `GLOBAL_CONFIG.read().unwrap()` are a different
matter — a poisoned `RwLock` is unrecoverable by definition and remains acceptable;
it is not covered by this rule.

## Orchestration-Layer Functions Must Be async

Execution functions in the orchestration layer (the `service/` directory, and any
function that performs orchestration duties, such as yell's `NotificationRouter`
methods) **must** be declared as `async fn`, even if the current implementation is
entirely synchronous.

- Reason one (compatibility): the orchestration layer will inevitably call async
  atomic operations (MQ publishing, async HTTP, etc.). Standardizing on `async`
  now avoids a signature change later that would cascade through every caller.
- Reason two (style consistency): all orchestration interfaces across the workspace
  share one shape, and handlers always call `service::xxx(...).await` — no more
  mental overhead from mixing sync and async.

✅ Correct (add `async` to the existing synchronous signature):

```rust
pub async fn new_user<'a>(
    user_to_creat: UserInputStream,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<String>> {
    // ...
}
```

Constraints and notes:

- This rule constrains only the orchestration layer's **signature shape**, not the
  underlying behavior: synchronous diesel operations in the model layer stay as
  they are — going `async` does not make them non-blocking. If truly not blocking
  the worker thread is needed, evaluate `web::block` separately; it is out of scope
  for this rule.
- The model layer (atomic operations) is not required to be async; it follows the
  sync/async nature of the client it uses.
- Existing synchronous service functions are legacy pending migration (see the
  migration plan in `doc/todo.md`): new orchestration functions must be async;
  when touching an existing synchronous function, migrate it in passing, and the
  migration must update all call sites (handlers switch to `.await` calls).

## No Commented-Out Dead Code in the Repository

Do not commit commented-out old implementations or debug snippets. History lives in
git, not in comments. See the comments guideline `doc/comments-EN.md` for details.

## Minimal-Change Principle

- Make only the changes the task requires: no drive-by refactoring, no cleanup
  unrelated to the task, no reformatting of untouched files.
- No speculative abstraction: before a second real call site exists, do not extract
  shared functions, generics, or config options. Consider consolidation only at
  three similar occurrences, and shared code should go into `share-lib` first —
  copy-paste across crates is forbidden.
- When an interface changes, you must update all its callers; but beyond adapting
  to the new interface, do not alter the callers' existing logic.

## Current Status & Exceptions

The following are real gaps between the codebase and the rules above. They are
legacy status pending convergence, not examples to follow:

- **clippy does not use `-D warnings`**: `build/clippy_all_ws.sh` runs
  `cargo clippy --workspace --all-targets`; warnings are currently recorded but do
  not block. The rule still requires zero new warnings in new code; once the
  baseline is cleared, enabling `-D warnings` in CI can be considered.
- **`pool.get().unwrap()` in middleware / services**: a failed pool checkout panics
  on the request path, violating the rule in section 3, but it is widespread today,
  e.g. `watchman-backend/src/middleware/auth_middleware.rs` (around lines 140,
  146, 151, 157, 167, 365, 486) and `watchman-backend/src/service/account_service.rs`,
  `access_service.rs`, among others. This is legacy debt to be gradually converted
  to `MailManErr` returns; new code must not copy this pattern.
- **Per-field `expect` in `config/server.rs`**: e.g.
  `.expect("Config path server_config:log_path (string) not found")` in
  `watchman-backend/src/config/server.rs`. Because `reload()` serves both startup
  and the `/api/reload` hot-reload endpoint (a request path), it does not strictly
  fit the boundary in section 3; it is a pending item and stays as-is for now.
- **Existing commented-out code**: `watchman-backend/src/main.rs` and
  `watchman-backend/src/config/server.rs` still contain commented-out fields and
  logic (e.g. the `pub_key_path` / `pri_key_path` lines), left over from a key-scheme
  transition, pending cleanup.
