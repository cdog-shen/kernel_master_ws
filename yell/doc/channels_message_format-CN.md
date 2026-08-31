# yell 各渠道消息格式与发送方式

本文档说明 yell 通知服务当前已实现的六个渠道（SMTP / Bark / Gotify / Teams /
Teams Hook / Webhook）的 HTTP（或 SMTP）发送方式、请求体格式、recipient 语义
与失败处理行为。所有内容以代码为准，关键处标注源码位置（文件:行号）。

> 重要变更：直接发送端点 `POST /api/notify/send` 已删除，模板发送
> （`POST /api/notify/template`）是唯一的发送路径；`recipients` 只接受别名
> （alias）字符串；模板已移除 `ntfy` 列与 `params_template` 字段，`teams`
> 列改名为 `teams_hook`（迁移 `2026-08-27-000001_template_channel_changes`）。
> alias `recipients` 元素的 `recipient` 字段已由纯 string 改为非空数组，
> 元素 schema 由各渠道定义，旧 string 格式会被 400 拒绝（迁移
> `2026-08-31-000001_recipient_string_to_array`）。

## 1. 总览：模板发送流程

### 1.1 recipients 仅支持别名

唯一发送入口 `POST /api/notify/template`（handler 见 `api/notify.rs:17-63`）
要求请求体携带：

- `template_name`：模板名（必填，handler 侧校验缺失返回 400）；
- `variables`：模板变量对象（缺省为空对象，`filter.rs:96-112`）；
- `recipients`：**只接受别名字符串**。传入数组会被明确拒绝
  （400，"'recipients' only supports an alias name string; raw recipient
  arrays are no longer accepted"），缺失或类型不符同样 400
  （`notify_service.rs:21-81`）。

别名查 `notification_aliases` 表展开为数组，元素解析为
`Vec<RecipientTarget>`（`channel_type` / `instance` /
`recipients: Vec<Value>`，`notify_service.rs:84-141`）：

```json
[{ "channel_type": "bark", "recipient": ["aBcDeFg..."], "instance": "default" }]
```

- `channel_type`：渠道标识，决定走哪个 Channel 实现，并对应模板中的同名
  JSONB 列；
- `recipient`：**非空数组**，元素 schema 由各渠道自行定义与校验——
  smtp / bark / gotify / teams / webhook 为 string（邮箱地址 / device key /
  app token / webhook URL，空串元素表示回退到实例配置），teams_hook 为
  对象（见第 6 节）；旧的纯 string 格式会被 400 拒绝；
- `instance`：渠道配置实例名，必填，用于从 `channel_configs` 表取该实例的
  `config_json`。

### 1.2 编排与 fan-out

- 编排层按名称取启用模板（`notify_service.rs:221-251`），并加载全部
  `is_enabled=true` 的渠道配置为嵌套表
  `channel_type -> instance_name -> config_json`
  （`notify_service.rs:170-218`），渠道发送时只查这张注入的表，不再自行查库。
  recipients 为空时不加载渠道配置、不产生发送任务（`api/notify.rs:44-52`）。
- `NotificationRouter` 注册六个渠道：smtp / bark / gotify / teams /
  teams_hook / webhook（`notification_router.rs:31-42`；`wecom`、`lark`、
  `dingtalk` 未实现）。
- 对每个 `RecipientTarget`，仅当模板中存在对应渠道列时才 `tokio::spawn`
  一个并发任务（`notification_router.rs:61-89`）；模板未配置该渠道列的
  目标被静默跳过。渲染已下沉到各渠道（见 1.4），router 只负责路由与
  任务编排。
- 每个被派发的目标产生一条独立的 `notification_records` 记录和一个
  `ChannelResult`；任务级失败以 `success=false, record_id=-1` 兜底呈现
  （`notification_router.rs:91-112`）。

### 1.3 统一骨架（channel.rs）

`Channel` trait 已拆分为四个方法（`channel.rs:74-115`）：`channel_type`、
`preflight`（默认无操作，供渠道校验配置与 recipient 元素 schema、初始化
惰性资源）、`render`（默认实现渲染一次，teams_hook 逐元素渲染）、`send`
（各渠道的核心外呼方法）。原来的 `dispatch_template` 已随本次重构拆分移除；
更早的 `prepare_request` / `dispatch`（直接发送钩子）已随直接发送路径一并
移除。

`deliver_template`（`channel.rs:145-175`）对每个 `RecipientTarget` 执行：

```
取注入配置(resolve_config) → preflight → render → create_record(status=pending)
    → send → update_record(sent|failed)
```

- 配置未找到：`"{渠道显示名} config not found"`，不创建记录，直接报错；
  `preflight` 失败（如 SMTP 初始化失败、recipient 元素 schema 非法）同样
  不创建记录、错误上抛。
- 外呼结果分两类（`DispatchError`，`channel.rs:56-63`）：
  - `Abort`：前置失败（参数/配置非法），记录保持 `pending`，错误直接上抛；
  - `Failed`：外呼已发起但失败，记录标记 `failed` 并写入 `error_msg`；
    成功则记录标记 `sent`，`sent_at` 写入当前本地时间
    （`channel.rs:178-206`、`channel.rs:315-347`）。
- `send` 对 recipient 元素逐个外呼，元素间互不阻断，全部尝试完毕后有错才
  返回 `Failed`（各元素错误以 "; " 汇总并标注失败元素，
  `collect_failures`，`channel.rs:233-240`）。
- 落库视图由渲染后 payload 构造（`record_view_from_payload`，
  `channel.rs:117-143`）：`title` ← payload.title，`body` ← payload.body 或
  payload.message，`url` ← payload.url，`template_id` 写入模板 id；记录的
  `recipient` 字段写入 recipients 数组的 JSON 序列化文本。

### 1.4 模板渲染

模板渲染规则（`template_render.rs:15-78`）：遍历 `smtp/bark/gotify/
teams_hook/webhook` 五列（`template_render.rs:11`），非 NULL 的列递归替换
其中字符串里的 `{{key}}`（非字符串变量按空串处理，
`template_render.rs:45-53`）。模板发送路径下渲染已下沉到各渠道：router 只
按 `get_channel_json` 取本渠道模板列（`template_render.rs:34-42`），实际
渲染在 `Channel::render` 中进行（默认实现 `channel.rs:93-104` 渲染一次；
teams_hook 逐元素渲染，见第 6 节）。全量渲染 `render_template`（产出
`HashMap<channel_type, payload>`）仅保留给模板预览（`render_example`，
`template_render.rs:81-87`）使用。模板已无 `ntfy` 列与 `params_template`
字段；原 `teams` 列已改名为 `teams_hook`（RENAME 保留存量数据，见
`migrations/2026-08-27-000001_template_channel_changes/up.sql`）。

## 2. SMTP（邮件，`channel_type = "smtp"`）

源码：`yell/src/services/mail/service.rs`

### 发送方式

- 协议 SMTP，使用 lettre 的 `AsyncSmtpTransport`（Tokio executor）。
- 配置结构 `SmtpConfig { host, port, username, password, from, use_tls }`
  （`channel_config.rs:41-48`）。
- `use_tls=true` 时走 `starttls_relay(host)`（STARTTLS），否则
  `builder_dangerous`（明文）（`mail/service.rs:55-66`）。认证用
  username/password `Credentials`。
- 连接复用：transport 按实例配置懒加载并缓存在
  `Mutex<HashMap<String, ...>>` 中，缓存 key 由
  host/port/username/password/use_tls 拼接（`mail/service.rs:16-70`），
  不同实例配置各自持有独立 transport，由 `preflight` 保证初始化
  （`mail/service.rs:131-139`）。
- 代码中未见 SMTP 超时配置。

### 消息格式（模板发送）

一封邮件，`From = config.from`，`To = 全部地址元素`，
`Subject = payload.subject`，正文 = `payload.body`；
`payload.content_type == "html"` 时 Content-Type 为 `text/html`，否则
`text/plain`（`mail/service.rs:153-163`）。预设模板 `smtp_html_alert`
（`migrations/2026-05-22-020000_refactor_template_schema/up.sql:88-95`）：

```json
{ "subject": "[{{level}}] {{service}} 告警", "content_type": "html",
  "body": "<h2>{{level}} — {{service}}</h2><p>{{message}}</p><p><a href=\"{{url}}\">查看详情</a></p>" }
```

### recipient 语义

元素为邮件地址 string，**数组中每个元素即一个 To 地址**；元素先经
`string_elements` 校验为 string，再去空白、去空串
（`mail/service.rs:83-95`）。清洗后无有效地址则报错
"No valid recipient addresses provided"。旧的 `,`/`;` 分隔符多地址写法
已移除，多地址一律用数组多元素表达。

### 失败处理

地址非法在 `preflight` 阶段中止（不创建记录、错误上抛）；配置/建信失败
为 `Abort`（不置 failed），只有 `transport.send` 失败才是
`Failed("SMTP send error: ...")`（`mail/service.rs:167-170`）。

## 3. Bark（iOS 推送，`channel_type = "bark"`）

源码：`yell/src/services/bark/service.rs`

### 发送方式

- HTTP POST JSON，URL = `{server_url}/push`（`server_url` 去掉尾部 `/` 拼接，
  `bark/service.rs:43`），配置 `BarkConfig { server_url, device_key? }`
  （`channel_config.rs:52-55`）。
- 无认证 header；device_key 放在请求体中而非 URL path。
- HTTP 走 `run_http_call`：同步 ureq 调用放进 `web::block` 线程池执行
  （`channel.rs:255-273`），代码中未见自定义超时或连接池配置（ureq 每次
  新建请求）。

### 消息格式（模板发送）

`send` 把渲染后的 payload 原样作为请求体，仅补上 `device_key`，对每个
recipient 元素各发一次（`bark/service.rs:33-64`）。预设模板 `urgent_alert`
的 bark 列（`2026-05-22-020000` up.sql:38）：

```json
{ "title": "[{{level}}] {{service}} 告警", "body": "{{message}}",
  "level": "timeSensitive", "sound": "alarm", "group": "{{group}}", "url": "{{url}}" }
```

即模板里写的 `level/sound/group/url` 等字段会原样进 Bark API 请求体，
可用字段完全由模板作者控制（对应 Bark API 的字段集）。

### recipient 语义

元素为 Bark device_key string。空串元素回退到配置中的默认 `device_key`；
两者都为空时请求体不带 `device_key`，即向该 Bark server 上的全体用户广播
（`bark/service.rs:46-54`）。

### 失败处理

全部元素尝试完毕后有错才返回 `Failed`（各元素错误以 "; " 汇总并标注
device_key）；配置 JSON 解析失败 → `Abort("Invalid Bark config: ...")`。

## 4. Gotify（自托管推送，`channel_type = "gotify"`）

源码：`yell/src/services/gotify/service.rs`

### 发送方式

- HTTP POST **form-urlencoded**（非 JSON），URL =
  `{server_url}/message?token={app_token}`（`gotify/service.rs:66-74`）。
  配置 `GotifyConfig { server_url, app_token }`（`channel_config.rs:59-62`）；
  认证方式是 query 里的 `token`。
- body 形如 `title=...&message=...&priority=...[&extras=...]`，手工拼接、
  未做 URL 编码（`gotify/service.rs:54-64`）——title/message 含 `&`、`=`
  等字符时可能被截断，代码中未见转义处理。

### 消息格式（模板发送）

从渲染后 payload 取 `title`、`message`、`priority`（u64，缺省 5）、`url`
（非空则封装为 `extras={"client::notification":{"click":{"url":...}}}`），
拼成 form 后对每个 recipient 元素各发一次（`gotify/service.rs:43-64`）。
预设模板 `urgent_alert` 的 gotify 列（up.sql:39）：

```json
{ "title": "[{{level}}] {{service}} 告警", "message": "{{message}}", "priority": 8, "url": "{{url}}" }
```

注意：`priority` 只接受 JSON 数字（`as_u64`），字符串形式的数字会被忽略
并回落到缺省 5。

### recipient 语义

元素为 Gotify 应用 token（app token）string。空串元素回退到配置中的
`app_token`（`gotify/service.rs:68-73`）；都为空则 `token=` 为空串发出，
鉴权必然失败。

### 失败处理

同 Bark：`run_http_call` 的 Ok → sent，全部元素尝试完毕后有错才返回
`Failed`（标注 app_token）；配置解析失败 →
`Abort("Invalid Gotify config: ...")`。

## 5. Teams（`channel_type = "teams"`）

源码：`yell/src/services/teams/service.rs`

> 注意：模板的 `teams` 列已改名为 `teams_hook`，渲染结果中不再存在
> `teams` 键，因此 router 虽已注册本渠道（`notification_router.rs:37`），
> 模板发送路径下 `channel_type = "teams"` 的目标实际会被静默跳过。
> 本节描述其实现行为，供后续恢复或迁移参考。

### 发送方式

- HTTP POST JSON 到 Teams Incoming Webhook URL。配置
  `TeamsConfig { webhook_url }`（`channel_config.rs:66-68`）；空串元素回退
  到配置的默认 URL（`teams/service.rs:95-100`）。
- 认证即 webhook URL 本身（无额外 header/token）。

### 消息格式（模板发送）

固定 MessageCard 格式（`teams/service.rs:43-92`）：`title` ← payload.title，
`text` ← payload.text 或 payload.body；`title/text/body/url` 以外的顶层
字段，若值是字符串/数字/布尔，自动收集进 `sections[0].facts`
（`{name, value}`）；`url` 非空时加 `potentialAction`（OpenUri
"查看详情"）。themeColor 固定 `0076D7`（不随优先级变化）。同一份
MessageCard 对每个 URL 元素各 POST 一次。

```json
{
  "@type": "MessageCard",
  "@context": "http://schema.org/extensions",
  "summary": "payload.title",
  "themeColor": "0076D7",
  "title": "payload.title",
  "text":  "payload.text 或 payload.body",
  "sections": [{ "facts": [{ "name": "...", "value": "..." }] }],
  "potentialAction": [{ "@type": "OpenUri", "name": "查看详情",
                        "targets": [{ "os": "default", "uri": "<url>" }] }]
}
```

### recipient 语义

元素为 Teams Incoming Webhook 完整 URL string。空串元素回退到配置的
`webhook_url`。

### 失败处理

同其他 HTTP 渠道：post_json 成功 → sent；全部元素尝试完毕后有错才返回
`Failed`（标注 URL）；配置解析失败 → `Abort("Invalid Teams config: ...")`。

## 6. Teams Hook（`channel_type = "teams_hook"`）

源码：`yell/src/services/teams_hook/service.rs`

### 发送方式与消息格式

- recipient 元素为**对象**而非 URL string：允许的 key 为
  `user` / `group_id` / `team_id` / `channel_id`（`ALLOWED_KEYS`，
  `teams_hook/service.rs:16`），均为可选但至少出现一个、值必须为 string，
  由 `preflight` 校验（`teams_hook/service.rs:31-54`）。
- 渲染逐元素进行（`render`，`teams_hook/service.rs:57-75`）：每个元素的
  字段 merge 进模板 variables（元素字段覆盖同名变量），再对模板
  `teams_hook` 列渲染出该元素专属的 payload；`payloads[i]` 与
  `recipients[i]` 一一对应。
- HTTP POST JSON 到**实例配置中的** `WebhookConfig { webhook_url }`
  （`send`，`teams_hook/service.rs:78-99`）：每个元素渲染出的 payload 原样
  各 POST 一次，URL 不随元素变化。
- 渲染后的 payload **原样**作为请求体 POST，不做任何字段加工——不会自动
  包装成 MessageCard，payload 结构（MessageCard / Adaptive Card 等）完全由
  模板作者控制。
- 预设模板中原 `teams` 列的数据（如 `urgent_alert` 的
  `{"title": ..., "text": ..., "url": ...}`）经列改名后落在 `teams_hook`
  列，将按此逻辑原样 POST；如需 Teams 原生卡片格式，需在模板中自行写全
  MessageCard 结构。

### recipient 语义

元素为对象 `{"user": "...", "group_id": "...", "team_id": "...",
"channel_id": "..."}`：四个 key 均可选但至少一个，值必须为 string；
每个对象的字段 merge 进模板 variables 后逐元素渲染。示例：

```json
{ "channel_type": "teams_hook", "instance": "teams-main",
  "recipient": [{"user": "zhangsan", "group_id": "G-001"}, {"user": "lisi"}] }
```

### 失败处理

逐元素 POST，全部尝试完毕后有错才返回 `Failed`（错误信息标注元素下标，
如 `element 0: TeamsHook request error: ...`）；配置解析失败 →
`Abort("Invalid TeamsHook config: ...")`。

## 7. Webhook（通用，`channel_type = "webhook"`）

源码：`yell/src/services/webhook/service.rs`

### 发送方式

- HTTP POST JSON 到 webhook URL。配置 `WebhookConfig { webhook_url }`
  （`channel_config.rs:72-74`）；空串元素回退到配置的默认 URL
  （`webhook/service.rs:45-50`）。
- 无认证 header；如目标需要鉴权，只能把 token 编进 URL 本身。

### 消息格式（模板发送）

渲染后的 payload **原样**作为请求体，对每个 URL 元素各 POST 一次，不做
任何字段加工（`webhook/service.rs:34-61`）。预设模板 `webhook_json`
（up.sql:78-85）：

```json
{ "title": "{{title}}", "body": "{{body}}", "level": "{{level}}" }
```

### recipient 语义

元素为目标 webhook 完整 URL string。空串元素回退到配置的 `webhook_url`。

### 失败处理

同其他 HTTP 渠道：成功 → sent；全部元素尝试完毕后有错才返回 `Failed`
（标注 URL）；配置解析失败 → `Abort("Invalid Webhook config: ...")`。

## 8. 对比速查表

| 渠道 | 协议/方法 | 认证 | recipient 元素 | 模板 payload 处理 | 特殊行为 |
|---|---|---|---|---|---|
| smtp | SMTP（STARTTLS/明文），lettre | username/password | 邮箱地址 string，每个元素一个 To | 取 subject/body/content_type | transport 按实例配置缓存复用 |
| bark | HTTP POST JSON `{server_url}/push` | 无（device_key 在 body） | device_key string；空串回退配置（再空则广播） | payload 原样 + device_key，逐元素发送 | — |
| gotify | HTTP POST form `{server_url}/message?token=` | query token（app token） | app token string；空串回退配置 | 取 title/message/priority/url，逐元素发送 | form 手工拼接未转义；priority 仅接受 JSON 数字 |
| teams | HTTP POST JSON webhook URL | URL 本身 | webhook URL string；空串回退配置 | MessageCard + 未知字段→facts，逐元素发送 | 模板已无 teams 列，当前不可达 |
| teams_hook | HTTP POST JSON webhook URL（实例配置） | URL 本身 | 对象 `{user, group_id, team_id, channel_id}`（至少一个 key） | 元素字段 merge 进 variables 后逐元素渲染 | 唯一按元素定制 payload 的渠道 |
| webhook | HTTP POST JSON webhook URL | 无 | webhook URL string；空串回退配置 | payload 完全原样发送，逐元素发送 | 完全由模板作者控制 payload 的通用通道 |

## 9. 备注与已知缺口

- 直接发送路径（`POST /api/notify/send`、`NotificationRequest` 直发、
  `prepare_request` 降级钩子、`format=json` 自定义 params）已整体移除；
  `NotificationRequest` 结构体现在仅作为落库视图使用（`channel.rs:19-31`、
  `channel.rs:117-143`），其 `mentions` 字段无任何读取方。
- alias `recipients` 元素的 `recipient` 已数组化：旧格式（纯 string）在
  alias 写入与发送解析两侧都会被 400 拒绝
  （`notify_service.rs:84-166`）；`notification_records.recipient` 落库的
  是 recipients 数组的 JSON 序列化文本（列已放宽为 TEXT）。
- `teams` 渠道仍注册在 router 中，但模板没有 `teams` 列，`channel_type =
  "teams"` 的目标会被静默跳过、不产生记录（见第 5 节）。
- 预设模板（`urgent_alert`、`normal_notify`、`all_channels_alert`）的原
  `teams` 列数据在列改名后由 `teams_hook` 渠道**原样 POST**，不会包装为
  MessageCard；如需卡片格式需改写模板内容。
- HTTP 渠道统一用 ureq 同步调用包裹在 `web::block` 中执行
  （`channel.rs:255-273`），代码中未见超时、重试或连接池的显式配置；发送
  结果仅区分 sent/failed，未解析各渠道响应体中的业务级错误（`run_http_call`
  只检查 HTTP 层成败）。
