# Logging Rules

## Purpose

Unify how every crate logs: which facility to use, where to initialize it, which
level fits which situation, and how the MailMan system relates to logging. The goal
is that each piece of information is logged exactly once, in a consistent format,
with levels controllable via configuration — avoiding duplicate logs and noise from
casual `println!` calls.

## Logging Facility

The facade is the `log` crate; the implementation is share-lib's own `ShareLogger`
(`share-lib/src/logger.rs`). It holds an `Arc<Mutex<File>>` and appends to the log
file, while also printing each line synchronously to the console via `println!`.
Every line is formatted as `{rfc3339 timestamp} - {LEVEL} - {target} - {msg}`
(`share-lib/src/logger.rs:53`).

Business code must use the macros exported by share-lib; using `println!`/`eprintln!`
for business logging is forbidden:

```rust
// share-lib/src/logger.rs
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => (info!($($arg)*));
}
// Also log_warn! / log_error! / log_debug! / log_trace!, one-to-one with log crate macros
```

Note: these macros expand to unqualified `info!`/`warn!`/etc., so the calling module
must also `use log::{info, warn, error, debug, trace};` (import whichever are used).
See `share-lib/src/data_structure.rs:1-2` for the required imports.

✅ Recommended:

```rust
use share_lib::log_info;
use log::info;

log_info!("DB Pool init");
```

❌ Forbidden:

```rust
println!("user {} login ok", username); // bypasses level control, no levelled file output
```

## Initialization

Every binary crate must call `logger::init_logger(log_path, log_level)` at the top of
`main`, right after the configuration has been loaded. Both arguments come from
`GLOBAL_CONFIG`. Internally, `init_logger` performs `log::set_boxed_logger` and
`log::set_max_level`; it may only be called once (a second call panics).
See `watchman-backend/src/main.rs:44-48`:

```rust
// watchman-backend/src/main.rs
// init logger
logger::init_logger(
    &server::GLOBAL_CONFIG.read().unwrap().log_path,
    &server::GLOBAL_CONFIG.read().unwrap().log_level,
);
```

`log_path` and `log_level` are configured in each crate's `*.template.toml`.
Valid `log_level` values are `ERROR` / `WARN` / `INFO` / `DEBUG` (any other value
falls back to `Trace`; see `share-lib/src/logger.rs:24-30`).

## Level Conventions

Levels must be chosen by semantics; raising a level just to make a log "more visible"
is forbidden:

- `log_error!` — an operation failed and needs human attention (DB write failure,
  external call error with no fallback).
- `log_warn!` — a recoverable anomaly (a failure before a successful retry, a missing
  config value that fell back to a default).
- `log_info!` — key business milestones (service startup, config reload completed,
  login succeeded). Low frequency.
- `log_debug!` / `log_trace!` — diagnostic details (request parameters, intermediate
  values). May be high-frequency; disable in production via `log_level`.

❌ Forbidden: emitting per-item data at `info` level or above inside loops or
hot paths.

## HTTP Access Logs

HTTP access logs (method, path, status code, latency) are produced uniformly by
`actix_web::middleware::Logger`, registered where each crate builds its `HttpServer`
(see `watchman-backend/src/main.rs:91`):

```rust
// watchman-backend/src/main.rs
// wrap default logger
.wrap(actix_web::middleware::Logger::default())
```

Handlers must not manually log the request line (method + path at info level), as
that duplicates the middleware output. Handlers only log business events themselves.

## MailMan and Logging

`MailManOk::new` / `MailManErr::new` log as a side effect of construction
(`share-lib/src/data_structure.rs`):

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

The resulting conventions (mandatory):

1. An error must be wrapped in `MailManErr` where it originates (the lowest layer
   that holds the error object); construction completes the logging. Upper layers
   of the call chain only propagate `Err(...)` and must not log the same error again.
2. Ordinary progress logging uses the `log_*!` macros, never MailMan.
3. Constructing and discarding a `MailManOk` just to log is forbidden — success-path
   progress information belongs in `log_info!`.

✅ Recommended:

```rust
// Bottom layer: construct where the error occurs, logging happens once
Err(msg) => Err(MailManErr::new(500, "Service: Login", Some(msg.1), 1)),
```

```rust
// Upper layers: propagate only, no re-logging
let user = login(req, pool)?; // or match and return Err(e) unchanged
```

❌ Forbidden:

```rust
// watchman-backend/src/service/account_service.rs:82-86 (existing counter-example, to be converged)
Ok(msg) => {
    MailManOk::new(            // return value discarded; only built for its info-log side effect
        200,
        "Service: Login - login time",
        Some(format!("Line changed: {msg}")),
    );
}
```

This case should be rewritten as `log_info!("Service: Login - login time, Line changed: {msg}")`.

## Current Status and Exceptions

- This repository does not use `tracing`, `metrics`, `env_logger`, or `log4rs`.
  All logging goes through the `log` facade plus `ShareLogger`; there are no
  structured fields or metrics collection. If `tracing` is introduced in the future,
  its compatibility with `ShareLogger` and the MailMan side effects must be evaluated first.
- `ShareLogger`'s `flush` is a no-op (`share-lib/src/logger.rs:70`), so a process
  crash could theoretically lose lines not yet flushed to disk. This risk is accepted
  for now; no rework is planned.
- `MailManOk::new` also logs its key at info level (`share-lib/src/data_structure.rs:14`),
  meaning every successful response leaves one info line. This is existing behavior
  and will not change in this phase; when writing log-auditing scripts, distinguish
  MailMan key logs from business `log_info!` calls.
- Legacy code contains the construct-and-discard `MailManOk` logging pattern
  (e.g. `watchman-backend/src/service/account_service.rs:82`, `:101`, `:107`).
  Per current-phase rules these are not bulk-refactored, but all new code must follow
  the "MailMan and Logging" section above.
- `watchman-backend/src/main.rs:42` prints the full configuration with `println!`
  before the logger is initialized. This is a startup debugging leftover and may
  remain, but new code must not imitate it.
