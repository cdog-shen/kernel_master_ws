# Yell

Yell is the notification service of the Kernel Master project.
It listens on port **9005** and dispatches notifications to multiple channels
(Bark / Gotify / Mail(SMTP) / Teams / Teams Hook / Webhook), with template
rendering, channel instance configuration, recipient aliases, and delivery
records persisted in PostgreSQL.

## Code Conventions

- All ORM-related operations are placed in the corresponding module under the ***model*** directory.

- All channel-sending and management logic belongs in the ***services*** directory.

    Each channel (`bark/`, `gotify/`, `mail/`, `teams/`, `teams_hook/`,
    `webhook/`) implements the `Channel` trait defined in
    `services/channel.rs` (`channel_type` / `preflight` / `dispatch_template`).
    `notification_router.rs` routes a request to the requested channel
    instances and records the delivery.

- API-response–related logic goes into the corresponding API module under the ***api*** directory.

    Input maps are parsed via `filter.rs` (`from map`); any missing fields are treated as `None`.
    All errors are returned through the MailMan system (`MailManErr` → `MailManErrResponser`).

## APIs

All endpoints are prefixed with `/api`. Except `/hey` and `/manage`, every
scope is wrapped by the `Authentication` middleware.

All success responses are wrapped in `MailManOk` (`{code, key, data}`),
including the query-style GET endpoints (`/template/get`, `/template/help`,
`/record/get`, `/channel/get`, `/alias/get`), whose payload sits in `data`.

### /hey

| Resource | Supported Methods | Purpose | Notes |
| :------: | :---------------: | :------ | :---- |
|    /     |    `GET` `POST`   | Health check | Public, no auth required |

### /manage

|     Resource      | Supported Methods | Purpose | Notes |
| :---------------: | :---------------: | :------ | :---- |
|  /refresh_master  |      `POST`       | Re-register this instance with watchman | Public, no auth required; implemented in `api/system_manage.rs` |

### /notify

|  Resource  | Supported Methods | Purpose | Notes |
| :--------: | :---------------: | :------ | :---- |
| /template  |      `POST`       | Send with a template | Requires `template_name` and `variables`; `recipients` only accepts an alias name string, which is resolved to concrete recipients via `notification_aliases` |

The former direct-send endpoint `POST /api/notify/send` has been removed;
template sending is the only send path now.

### /template

| Resource | Supported Methods | Purpose | Notes |
| :------: | :---------------: | :------ | :---- |
|   /get   |       `GET`       | List templates | Query-string filters supported; `is_enabled` takes `true`/`false` strings |
|  /help   |    `GET` `POST`   | Mock-render example of a template | See [Template help](#template-help) below |
|   /new   |      `POST`       | Create a template | Per-channel bodies: `smtp` / `bark` / `gotify` / `teams_hook` / `webhook` |
| /update  |      `POST`       | Update a template | Requires `id` |
| /delete  |      `POST`       | Delete a template | Requires `id` |

#### Template help

`GET /api/template/help?name=<template_name>` or
`POST /api/template/help` with JSON body `{"name": "<template_name>"}`.

Returns the template mock-rendered per channel as JSON (wrapped in
`MailManOk`, payload in `data`). Rendering rules
(implemented in `services/template_render.rs::render_example`):

- every `{{var}}` placeholder is filled with the string `"TEST"`;
- numeric / boolean literals in the template are kept as-is;
- only the channel columns actually configured on the template appear in the
  response.

Error responses: `400` when `name` is missing or not a string; `404` when the
template does not exist or is disabled.

Example — `POST /api/template/help` with `{"name": "webhook_json"}` (a preset
template installed by the migrations, whose only configured channel is
`webhook`):

```json
{
    "code": 200,
    "key": "Template example rendered",
    "data": {
        "webhook": {
            "title": "TEST",
            "body": "TEST",
            "level": "TEST"
        }
    }
}
```

### /record

| Resource | Supported Methods | Purpose | Notes |
| :------: | :---------------: | :------ | :---- |
|   /get   |       `GET`       | Query notification delivery records | Query-string filters supported |

### /channel

| Resource | Supported Methods | Purpose | Notes |
| :------: | :---------------: | :------ | :---- |
|   /get   |       `GET`       | List channel configurations | |
| /update  |      `POST`       | Update a channel instance config | Requires `channel_type`, `name` (instance name) and `config_json` |

### /alias

| Resource | Supported Methods | Purpose | Notes |
| :------: | :---------------: | :------ | :---- |
|   /get   |       `GET`       | List all aliases | |
|   /new   |      `POST`       | Create an alias | Requires `name` and `recipients` |
| /update  |      `POST`       | Update an alias | Requires `id` |
| /delete  |      `POST`       | Delete an alias | Requires `id` |

## Supported Channels

| Channel | `channel_type` | Notes |
| :-----: | :------------: | :---- |
|  Bark   |     `bark`     | iOS push via Bark server |
| Gotify  |    `gotify`    | Self-hosted push via Gotify server |
|  Mail   |     `smtp`     | Email via SMTP (lettre) |
|  Teams  |    `teams`     | Microsoft Teams MessageCard via Incoming Webhook; still registered, but templates no longer have a `teams` column (renamed to `teams_hook`), so it is currently unreachable via template sending |
| Teams Hook | `teams_hook` | Microsoft Teams via Incoming Webhook; same logic as `webhook` — the rendered payload is POSTed as-is |
| Webhook |    `webhook`   | Generic HTTP webhook; the rendered template payload is POSTed as-is |

Multiple instances per channel type are supported (multi-instance channel
configuration, keyed by `channel_type` + instance `name`).

## Deployment & Dependencies

- **System Libraries**

    - openssl
    - libpq

- **Required Services**

    - PostgreSQL

### Configuration File

Copy `yell_backend.template.toml` to `yell_backend.toml` in the working directory
and adjust it. Main options:

```toml
[server_config]
# Logging
log_path = "logs/yell.log"
log_level = "INFO"
clear_log = "True"
# Server params
listen_addr = "127.0.0.1"
listen_port = 9005
workers = 1
# Instance registration (reported to watchman)
uuid = "550e8400-e29b-41d4-a716-446655440000"
register_name = "yell"
master_addr = "127.0.0.1"
master_port = 8000
# CORS & auth bypass
allowed_origin_list = ["http://localhost:3000", "http://127.0.0.1:3000"]
authenticate_bypass = ["/api/refresh_master"]

# Database
[db_config]
db_str = "postgres://postgres:password@localhost:5432/yell"
```

### Database Migration

Migrations live in `migrations/` and are applied manually with the diesel CLI:

```sh
diesel migration run
```

### Run

```sh
# build from the workspace root
cargo build --release -p yell
# run with yell_backend.toml present in the working directory
./target/release/yell
```

Or via docker-compose (workspace root): `docker-compose --profile run up -d yell`.
