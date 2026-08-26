# Error Handling Rules

## Purpose

This document unifies how errors are expressed across all crates: every error must
flow through the MailMan message system and be mapped to an HTTP response by
`MailManErrResponser`. The goals are predictable error-code semantics, no missing
error logs, and no duplicated shared code. Audience: project developers and AI
coding assistants.

## The MailMan Message System

Success and failure messages are defined in `share-lib/src/data_structure.rs`:

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

Both constructors carry a logging side effect: `MailManOk::new` calls
`log_info!("{}", key)` internally; `MailManErr::new` dispatches the log level by
`level` — `0` logs warn, `1` logs error, and any other value logs trace (note the
field comment says "fatal", which does not match the implementation; follow the
implementation):

```rust
match level {
    0 => log_warn!("{}, {:?}", key, error_obj),
    1 => log_error!("{}, {:?}", key, error_obj),
    _ => log_trace!("{}, {:?}", key, error_obj),
};
```

Rules:

- You MUST leverage this side effect for error logging: pick the correct `level`
  once when constructing a `MailManErr`; do not add a separate log statement.
- You MUST NOT construct a `MailManOk` just to log and then discard it. To emit an
  info log, call the `log_info!` macro directly. This pattern exists in current
  code (see "Current State and Exceptions" below); new code must not repeat it.

## The Three-Layer Error Flow

Errors travel up through three layers — model → service → api — with a different
shape at each layer.

### Model layer: return `Result<T, (u8, String)>`

The model layer is HTTP-agnostic and expresses errors as an
`(error code, error message)` tuple. Error codes are defined as constants at the
top of the model file; see `watchman-backend/src/model/user.rs`:

```rust
static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

// ...
Err(NotFound) => Err((BAD_REQUEST_CODE, format!("NotFound {user_name}."))),
Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
```

Convention: `0` means unknown/internal error (`UNKNOW_ERROR_CODE`; "UNKNOW" in
the constant name is a historical misspelling — keep it, do not rename), and `1`
means request-level error (`BAD_REQUEST_CODE`).

### Service layer: return `Result<MailManOk<'a, T>, MailManErr<'a, String>>`

The service layer translates the model layer's `(u8, String)` into a `MailManErr`
carrying HTTP semantics, naming `key` after the `"Service: <api name> [- <step>]"`
convention. See `watchman-backend/src/service/account_service.rs`:

```rust
match TokenModel::delete(&user_info.username.unwrap(), &mut pool.get().unwrap()) {
    Ok(msg) => Ok(MailManOk::new(200, "Service: Logout", Some(msg))),
    Err(msg) => match msg.0 {
        1 => Err(MailManErr::new(400, "Service: Logout", Some(msg.1), 1)),
        _ => Err(MailManErr::new(500, "Service: Logout", Some(msg.1), 1)),
    },
}
```

Rules:

- You MUST perform the model-code → HTTP-code translation at this layer:
  `1` → `400`, and `0` (plus any other unknown value) → `500`.
- The 4th argument of `MailManErr::new` (`level`): use `1` (error) for errors that
  warrant alerting; expected failures (e.g. a wrong login password) may use
  `0` (warn).

### API layer: map with `MailManErrResponser::mapping_from_mme`

API handlers return `Result<HttpResponse, MailManErrResponser>`; service errors
are converted via `mapping_from_mme`, and actix-web serializes the result into a
JSON response automatically. See `watchman-backend/src/api/account_manage.rs`:

```rust
match account_service::login(user_info, &pool) {
    Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
    Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
}
```

Errors raised by the API layer itself (e.g. request-payload parsing) construct a
`MailManErr` and map it immediately:

```rust
let user_info = user::UserInputStream::from_map(map.0).map_err(|e| {
    MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
})?;
```

## Error-Code to HTTP-Semantics Mapping

The mapping is defined in `share-lib/src/err_mapping.rs` and is fixed:

| code | HTTP semantics |
|---|---|
| 400 | BAD_REQUEST |
| 401 | UNAUTHORIZED |
| 403 | FORBIDDEN |
| 404 | NOT_FOUND |
| 501 | NOT_IMPLEMENTED |
| 503 | SERVICE_UNAVAILABLE |
| all others | INTERNAL_SERVER_ERROR (500) |

Note: this file maintains the same table in two `match` expressions — the one in
`mapping_from_mme` decides the `key` field of the response JSON, and
`ResponseError::status_code()` decides the HTTP status code. When changing the
mapping, both must be updated together; editing only one is forbidden.

New errors MUST first reuse a code from this table; introduce a new code only when
no existing semantics fits, and when you do, update both `match` expressions and
this document at the same time.

## Where MailManErrResponser Lives and How to Import It

`MailManErrResponser` lives in share-lib and is compiled only under the `web`
feature (see `share-lib/Cargo.toml`: `web = ["dep:actix-web"]`). Crates that need
web capability MUST import it like this:

```toml
share-lib = { workspace = true, features = ["web"] }
```

```rust
use share_lib::err_mapping::MailManErrResponser;
```

Historically, `err_mapping.rs` was duplicated byte-for-byte in 6 crates; it has
since been consolidated. Copying this file or a variant of it into any crate is
forbidden for any reason. If the mapping behavior genuinely needs to change,
modify `share-lib/src/err_mapping.rs` and update all callers accordingly.

## Field-Filling Rules

- `code`: prefer reusing a value from the semantics table above.
- `key`: a fixed, human-readable phrase that identifies where and at which step
  the error occurred, following existing conventions (`"Service: Login - Auth"`
  at the service layer, `"Bad Request"` at the API layer). Dynamic data MUST NOT
  be concatenated into `key` — dynamic information belongs in `msg`.
- `msg`: carries contextual details (which record, which field, the underlying
  error text), e.g. `format!("NotFound {user_name}.")`. It may be `None` — the
  mapping falls back to `"No more Msg"` — but providing it is recommended.

## Current State and Exceptions

The following gaps are recorded as-is; new code must not repeat them, and existing
code will be brought into line gradually:

- The `MailManErr` field comment says `0-2 as warn error fatal`, but the
  implementation logs "other values" as trace, not fatal
  (`share-lib/src/data_structure.rs`). Follow the code behavior.
- The model-layer constant `UNKNOW_ERROR_CODE` is a misspelling of "UNKNOWN";
  it is kept as-is for compatibility.
- In `login` of `watchman-backend/src/service/account_service.rs`, there are
  cases of "constructing and discarding a `MailManOk` purely for its logging side
  effect" (updating last-login time and saving the token), which violates the
  rule in the second section above and is a historical leftover.
- The error returns in each crate's `middleware/auth_middleware.rs` have diverged
  (they depend on their own models) and have not been consolidated into share-lib;
  any change to them requires a per-crate synchronized review.
