# yell alias recipient 数组化改造 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 `notification_aliases.recipients` 元素的 `recipient` 字段从纯 string 改为非空数组，渲染下沉到各渠道，teams_hook 支持按元素 merge variables 后逐元素渲染发送。

**Architecture:** `Channel` trait 从单一 `dispatch_template` 拆为 `preflight` / `render` / `send` 三步，所有渠道同构；编排层只校验"数组非空"，元素 schema 由渠道校验；一条三元组一条发送记录（渠道内聚合语义）。

**Tech Stack:** Rust / actix-web / diesel(Postgres, JSONB) / serde_json / lettre。

**Spec:** `docs/superpowers/specs/2026-08-31-yell-alias-recipient-array-design.md`

## Global Constraints

- 本期规约：**不补单测**；验证手段为 `cargo check -p yell`、`cargo clippy -p yell --all-targets`、`build/fmt_all_ws.sh -- --check` 及 migration 实跑。
- 错误必须走 MailMan 体系（`MailManErr` / `MailManErrResponser`），渠道外呼错误走 `DispatchError::Abort` / `Failed` 语义。
- 依赖版本只在根 `Cargo.toml` 改；本计划不新增任何依赖。
- 每个 migration 目录必须有 `up.sql` 和 `down.sql`。
- 公共代码优先进 share-lib；本计划改动全部限定在 `yell/` crate 内。
- 提交粒度：每个 Task 一个 commit，commit message 用英文（仓库惯例）。

---

### Task 1: Migration — recipient string → array + records.recipient 放宽为 TEXT

**Files:**
- Create: `yell/migrations/2026-08-31-000001_recipient_string_to_array/up.sql`
- Create: `yell/migrations/2026-08-31-000001_recipient_string_to_array/down.sql`
- Modify: `yell/src/model/schema.rs:35`（`recipient -> Varchar` → `recipient -> Text`）

**Interfaces:**
- Consumes: 无
- Produces: DB 层新格式——alias `recipients` 元素的 `recipient` 一律为数组；`notification_records.recipient` 为 TEXT（schema.rs 中 `recipient -> Text`，无 `max_length` 属性）。

- [ ] **Step 1: 写 up.sql**

```sql
-- recipient 字段由纯 string 迁移为数组（存量 string 包装为单元素数组，保序）
-- 同时将 notification_records.recipient 放宽为 TEXT（数组 JSON 序列化后可能超过 512 字符）

UPDATE notification_aliases a
SET recipients = (
    SELECT jsonb_agg(
               jsonb_set(elem, '{recipient}', jsonb_build_array(elem->'recipient'))
               ORDER BY ord
           )
    FROM jsonb_array_elements(a.recipients) WITH ORDINALITY AS t(elem, ord)
);

ALTER TABLE notification_records ALTER COLUMN recipient TYPE TEXT;
```

- [ ] **Step 2: 写 down.sql**

```sql
-- 回滚：单元素数组还原为 string；多元素数组取第一个元素（有损，仅用于回滚兜底）
-- notification_records.recipient 改回 VARCHAR(512)（若已有超长数据此步会失败，需手工处理）

UPDATE notification_aliases a
SET recipients = (
    SELECT jsonb_agg(
               jsonb_set(elem, '{recipient}',
                   CASE WHEN jsonb_typeof(elem->'recipient') = 'array'
                        THEN elem->'recipient'->0
                        ELSE elem->'recipient'
                   END)
               ORDER BY ord
           )
    FROM jsonb_array_elements(a.recipients) WITH ORDINALITY AS t(elem, ord)
);

ALTER TABLE notification_records ALTER COLUMN recipient TYPE VARCHAR(512);
```

- [ ] **Step 3: 同步 schema.rs**

`yell/src/model/schema.rs` 中 `notification_records` 表的 recipient 字段：

```rust
// 改前：
        #[max_length = 512]
        recipient -> Varchar,
// 改后：
        recipient -> Text,
```

- [ ] **Step 4: 在 yell_test 上执行并验证回滚**

```bash
cd yell
diesel migration run --database-url "postgres://be:Virtuos%40BE-SHA@10.72.2.12:5432/yell_test"
# 抽查改写结果：recipient 应全部为数组
diesel migration redo --database-url "postgres://be:Virtuos%40BE-SHA@10.72.2.12:5432/yell_test"
```

预期：`run` 成功；redo（down+up）成功。抽查 SQL（可用 psql 或任何 PG 客户端）：
`SELECT jsonb_typeof(elem->'recipient') FROM notification_aliases, jsonb_array_elements(recipients) elem;` 全部返回 `array`。

- [ ] **Step 5: Commit**

```bash
git add yell/migrations/2026-08-31-000001_recipient_string_to_array yell/src/model/schema.rs
git commit -m "feat(yell): migrate alias recipient to array, widen records.recipient to TEXT"
```

---

### Task 2: template_render — 暴露单渠道渲染与模板列访问

**Files:**
- Modify: `yell/src/services/template_render.rs:15-39`

**Interfaces:**
- Consumes: 无
- Produces:
  - `pub fn render_channel(template_json: &Value, variables: &Map<String, Value>) -> Value` — 渲染单个渠道 JSONB 列；
  - `pub fn get_channel_json<'t>(template: &'t NotificationTemplate, col: &str) -> Option<&'t Value>` — 由私有改为 pub，router 按 channel_type 取模板列。

- [ ] **Step 1: 修改 template_render.rs**

`render_template` 改为复用 `render_channel`；新增 pub 函数；`get_channel_json` 去私有：

```rust
/// 渲染模板，返回 HashMap<channel_type, rendered_payload_json>
/// 只渲染模板中已配置的渠道字段
pub fn render_template(
    template: &NotificationTemplate,
    variables: &Map<String, Value>,
) -> HashMap<String, Value> {
    let mut result = HashMap::new();
    for col in CHANNEL_COLUMNS {
        if let Some(json_val) = get_channel_json(template, col) {
            result.insert(col.to_string(), render_channel(json_val, variables));
        }
    }
    result
}

/// 渲染单个渠道 JSONB 列中的 {{var}} 占位符
pub fn render_channel(template_json: &Value, variables: &Map<String, Value>) -> Value {
    render_value(template_json, variables)
}

/// 获取指定渠道的 JSONB 字段
pub fn get_channel_json<'t>(template: &'t NotificationTemplate, col: &str) -> Option<&'t Value> {
    match col {
        "smtp" => template.smtp.as_ref(),
        "bark" => template.bark.as_ref(),
        "gotify" => template.gotify.as_ref(),
        "teams_hook" => template.teams_hook.as_ref(),
        "webhook" => template.webhook.as_ref(),
        _ => None,
    }
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo check -p yell`
预期：通过（此时新函数尚未被调用，可能有 dead_code 警告，忽略，Task 3 会接入）。

- [ ] **Step 3: Commit**

```bash
git add yell/src/services/template_render.rs
git commit -m "feat(yell): expose render_channel and get_channel_json for per-channel rendering"
```

---

### Task 3: 核心重构 — trait 三步拆分 + 全部渠道适配

本任务是一个原子变更：trait 签名变化会同时波及 channel.rs / router / notify_service / 6 个渠道实现，必须一起改完才能编译。

**Files:**
- Modify: `yell/src/services/channel.rs`（trait、`deliver_template`、新增 `RecipientTarget` 与辅助函数）
- Modify: `yell/src/services/notification_router.rs`（去掉预渲染，透传模板列 + variables）
- Modify: `yell/src/services/notify_service.rs`（三元组改 `RecipientTarget`，数组校验，新增 `validate_recipients_shape`）
- Modify: `yell/src/services/mail/service.rs`
- Modify: `yell/src/services/bark/service.rs`
- Modify: `yell/src/services/gotify/service.rs`
- Modify: `yell/src/services/webhook/service.rs`
- Modify: `yell/src/services/teams_hook/service.rs`
- Modify: `yell/src/services/teams/service.rs`

**Interfaces:**
- Consumes: `template_render::{render_channel, get_channel_json}`（Task 2）
- Produces:
  - `pub struct RecipientTarget { pub channel_type: String, pub instance: String, pub recipients: Vec<Value> }`（channel.rs）
  - `pub fn string_elements<'r>(recipients: &'r [Value], prefix: &str) -> Result<Vec<&'r str>, String>`（channel.rs）
  - `pub fn collect_failures(errors: Vec<String>) -> Result<(), DispatchError>`（channel.rs）
  - `DispatchError::into_msg(self) -> String`（channel.rs）
  - trait 新签名：`preflight(config, recipients) -> Result<(), String>`、`render(recipients, template_json, variables) -> Result<Vec<Value>, DispatchError>`（默认实现单渲染）、`send(config, recipients, payloads) -> Result<(), DispatchError>`
  - `pub async fn deliver_template(channel, target, template_json, variables, template_id, channel_configs, pool) -> Result<ChannelResult, String>`
  - `notify_service::validate_recipients_shape(value: &Value) -> Result<(), String>`（Task 4 消费）

- [ ] **Step 1: 重写 channel.rs 的共享部分**

imports 顶部把 `serde_json::Value` 改为 `serde_json::{Map, Value}`，并新增 `RecipientTarget`（放在 `ChannelConfigs` 定义之后）：

```rust
/// 单个接收目标：alias recipients 数组中一个元素解析后的结果
/// recipients 为原始 JSON 元素数组，元素 schema 由各渠道自行定义与校验
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipientTarget {
    pub channel_type: String,
    pub instance: String,
    pub recipients: Vec<Value>,
}
```

`DispatchError` 增加：

```rust
impl DispatchError {
    /// 提取错误文本（元素级失败聚合时使用）
    pub fn into_msg(self) -> String {
        match self {
            DispatchError::Abort(m) | DispatchError::Failed(m) => m,
        }
    }
}
```

trait 改为：

```rust
/// 渠道 trait —— 所有通知渠道必须实现
/// 渠道只负责"校验/渲染/外呼"本身；配置获取与通知记录生命周期由本模块的
/// 统一骨架（deliver_template）管理
#[async_trait]
pub trait Channel: Send + Sync {
    /// 渠道类型标识
    fn channel_type(&self) -> &'static str;

    /// 外呼前置准备（默认无操作）
    /// 在创建通知记录之前调用；返回 Err 时中止发送，不创建记录。
    /// 渠道在此校验注入的配置、recipient 元素 schema、初始化惰性资源（如 SMTP transport）
    async fn preflight(&self, config: &Value, recipients: &[Value]) -> Result<(), String> {
        let _ = (config, recipients);
        Ok(())
    }

    /// 渲染本渠道 payload（默认实现：与 recipient 元素无关，渲染一次）
    /// template_json 为本渠道在模板中的 JSONB 列内容；
    /// 返回的 payloads[i] 与 recipients[i] 按下标一一对应（默认实现恒为 1 个）
    fn render(
        &self,
        recipients: &[Value],
        template_json: &Value,
        variables: &Map<String, Value>,
    ) -> Result<Vec<Value>, DispatchError> {
        let _ = recipients;
        Ok(vec![crate::services::template_render::render_channel(
            template_json,
            variables,
        )])
    }

    /// 执行外呼 —— 各渠道的核心发送方法
    /// config 为按 channel_type + instance_name 从注入配置中取出的 config_json；
    /// 元素间互不阻断，全部尝试完毕后有错才返回 Failed（错误信息汇总并标注失败元素）
    async fn send(
        &self,
        config: &Value,
        recipients: &[Value],
        payloads: &[Value],
    ) -> Result<(), DispatchError>;
}
```

删除旧 `dispatch_template` 方法。`string_elements` / `collect_failures` 加在 `channel_display_name` 之前：

```rust
/// 提取 string 类型的 recipient 元素；任一元素非 string 时返回错误信息
/// prefix 为渠道显示名（如 "Bark"）。空串元素原样保留，语义由各渠道解释
/// （bark/gotify/webhook/teams 中空串表示"回退到实例配置"）
pub fn string_elements<'r>(recipients: &'r [Value], prefix: &str) -> Result<Vec<&'r str>, String> {
    recipients
        .iter()
        .map(|v| {
            v.as_str()
                .ok_or_else(|| format!("{} recipient elements must be strings", prefix))
        })
        .collect()
}

/// 汇总元素级失败：无错误返回 Ok，否则 Failed（"; " 连接全部错误）
pub fn collect_failures(errors: Vec<String>) -> Result<(), DispatchError> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(DispatchError::Failed(errors.join("; ")))
    }
}
```

`deliver_template` 重写（替换旧的整个函数）：

```rust
/// 统一发送流程骨架（模板发送）：
/// 取注入的配置 → preflight（校验配置与元素 schema）→ render →
/// 创建记录(pending) → send → update_record(sent/failed)
pub async fn deliver_template(
    channel: &Arc<dyn Channel>,
    target: &RecipientTarget,
    template_json: &Value,
    variables: &Map<String, Value>,
    template_id: Option<i32>,
    channel_configs: &ChannelConfigs,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<ChannelResult, String> {
    let config = resolve_config(channel_configs, channel.channel_type(), &target.instance)?;
    channel.preflight(config, &target.recipients).await?;
    let payloads = channel
        .render(&target.recipients, template_json, variables)
        .map_err(DispatchError::into_msg)?;

    let recipient_text = serde_json::to_string(&target.recipients)
        .map_err(|e| format!("Failed to serialize recipients: {}", e))?;
    let record_id = create_record(
        channel.channel_type(),
        &recipient_text,
        &record_view_from_payload(&payloads[0], template_id),
        pool,
    )
    .await?;

    let outcome = channel.send(config, &target.recipients, &payloads).await;
    finalize_delivery(outcome, channel.channel_type(), record_id, pool).await
}
```

- [ ] **Step 2: 重写 notification_router.rs 的 send_with_template**

imports 中把 `deliver_template` 行改为引入 `RecipientTarget`：

```rust
use crate::services::channel::{
    Channel, ChannelConfigs, ChannelResult, RecipientTarget, deliver_template,
};
```

`send_with_template` 签名中 `recipients: Vec<(String, String, String)>` 改为 `recipients: Vec<RecipientTarget>`，函数体替换为：

```rust
        let template_id = template.id;
        let mut tasks = Vec::new();

        for target in recipients {
            // 只向模板中配置了对应字段的渠道发送
            if let Some(template_json) =
                template_render::get_channel_json(&template, &target.channel_type)
            {
                if let Some(channel) = self.channels.get(&target.channel_type) {
                    let channel = Arc::clone(channel);
                    let template_json = template_json.clone();
                    let variables = variables.clone();
                    let configs = channel_configs.clone();
                    let pool = pool.clone();

                    let task = tokio::spawn(async move {
                        deliver_template(
                            &channel,
                            &target,
                            &template_json,
                            &variables,
                            Some(template_id),
                            &configs,
                            &pool,
                        )
                        .await
                    });

                    tasks.push(task);
                }
            }
        }
```

结果收集部分（`for task in tasks { ... }` 到函数尾）保持不变。删除函数开头原有的 `let rendered = template_render::render_template(...)` 与相关注释。函数文档注释改为：

```rust
    /// 使用模板发送 — 只向模板中有对应字段的渠道发送
    ///
    /// template: 由编排层按名称查询后注入的模板对象
    /// channel_configs: 由编排层注入的渠道配置（channel_type -> instance_name -> config_json）
    /// 渲染已下沉到各渠道（Channel::render），本函数只负责路由与任务编排
```

- [ ] **Step 3: 修改 notify_service.rs**

imports 增加 `use crate::services::channel::RecipientTarget;`。
`resolve_recipients` 返回类型改为 `Result<Vec<RecipientTarget>, MailManErr<'a, String>>`。
`parse_recipients_array` 替换为：

```rust
/// 解析 recipients JSON 数组为 Vec<RecipientTarget>
fn parse_recipients_array(
    arr: &[Value],
) -> Result<Vec<RecipientTarget>, MailManErr<'static, String>> {
    let mut recipients = Vec::new();
    for v in arr {
        let obj = v.as_object().ok_or_else(|| {
            MailManErr::new(
                400,
                "Bad Request",
                Some("Each recipient must be an object".to_string()),
                1,
            )
        })?;
        let channel_type = obj
            .get("channel_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing 'channel_type' in recipient".to_string()),
                    1,
                )
            })?;
        let instance = obj
            .get("instance")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing 'instance' in recipient".to_string()),
                    1,
                )
            })?;
        let recipient_arr = obj
            .get("recipient")
            .and_then(|v| v.as_array())
            .filter(|a| !a.is_empty())
            .ok_or_else(|| {
                MailManErr::new(
                    400,
                    "Bad Request",
                    Some(
                        "Field 'recipient' must be a non-empty array (plain string recipients are no longer supported)"
                            .to_string(),
                    ),
                    1,
                )
            })?;
        recipients.push(RecipientTarget {
            channel_type: channel_type.to_string(),
            instance: instance.to_string(),
            recipients: recipient_arr.clone(),
        });
    }
    Ok(recipients)
}

/// 校验 recipients 字段的结构（alias 写入侧使用）：
/// 顶层数组、每个元素为对象、channel_type/instance 为 string、recipient 为非空数组
pub fn validate_recipients_shape(value: &Value) -> Result<(), String> {
    let arr = value
        .as_array()
        .ok_or_else(|| "'recipients' must be an array".to_string())?;
    for v in arr {
        let obj = v
            .as_object()
            .ok_or_else(|| "Each recipient must be an object".to_string())?;
        for key in ["channel_type", "instance"] {
            if obj.get(key).and_then(|v| v.as_str()).is_none() {
                return Err(format!("Missing or non-string '{}' in recipient", key));
            }
        }
        match obj.get("recipient").and_then(|v| v.as_array()) {
            Some(a) if !a.is_empty() => {}
            _ => {
                return Err(
                    "Field 'recipient' must be a non-empty array".to_string(),
                );
            }
        }
    }
    Ok(())
}
```

文件顶部文档注释第 1 行不变；`resolve_recipients` 的 `Vec<(String, String, String)>` 相关注释同步改为 `Vec<RecipientTarget>`。

- [ ] **Step 4: 重写 mail/service.rs（smtp）**

删除 `parse_addresses`（分隔符 hack 退役）。`build_email` 签名中 `addresses: &[&str]` 不变。trait impl 改为：

```rust
#[async_trait]
impl Channel for SmtpChannel {
    fn channel_type(&self) -> &'static str {
        "smtp"
    }

    /// 发送前确保 SMTP transport 已初始化，且全部收件地址合法
    async fn preflight(&self, config: &Value, recipients: &[Value]) -> Result<(), String> {
        self.init_transport(config).await?;
        let addresses = Self::address_elements(recipients)?;
        for addr in &addresses {
            Mailbox::from_str(addr)
                .map_err(|e| format!("Invalid recipient address '{}': {}", addr, e))?;
        }
        Ok(())
    }

    async fn send(
        &self,
        config: &Value,
        recipients: &[Value],
        payloads: &[Value],
    ) -> Result<(), DispatchError> {
        let smtp_config: SmtpConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid SMTP config: {}", e)))?;

        let transport = self.transport(&Self::transport_key(&smtp_config)).await?;
        let addresses = Self::address_elements(recipients).map_err(DispatchError::Abort)?;

        let payload = &payloads[0];
        let subject = payload
            .get("subject")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let body = payload.get("body").and_then(|v| v.as_str()).unwrap_or("");
        let content_type = if payload.get("content_type").and_then(|v| v.as_str()) == Some("html") {
            ContentType::TEXT_HTML
        } else {
            ContentType::TEXT_PLAIN
        };

        let email = Self::build_email(&smtp_config.from, &addresses, subject, content_type, body)?;

        match transport.send(email).await {
            Ok(_) => Ok(()),
            Err(e) => Err(DispatchError::Failed(format!("SMTP send error: {}", e))),
        }
    }
}
```

`SmtpChannel` impl 块中新增（替代 `parse_addresses`）：

```rust
    /// 提取并清洗收件地址元素（string 元素，去空白、去空串；全空报错）
    fn address_elements(recipients: &[Value]) -> Result<Vec<&str>, String> {
        let elements = string_elements(recipients, "SMTP")?;
        let addresses: Vec<&str> = elements
            .into_iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if addresses.is_empty() {
            return Err("No valid recipient addresses provided".to_string());
        }
        Ok(addresses)
    }
```

imports：`use crate::services::channel::{Channel, DispatchError, string_elements};`

- [ ] **Step 5: 重写 bark/service.rs**

```rust
use async_trait::async_trait;
use serde_json::{Value, json};
use share_lib::infrastructure::http_client;

use crate::model::channel_config::BarkConfig;
use crate::services::channel::{
    Channel, DispatchError, collect_failures, run_http_call, string_elements,
};

/// Bark 推送渠道实现
pub struct BarkChannel;

impl BarkChannel {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Channel for BarkChannel {
    fn channel_type(&self) -> &'static str {
        "bark"
    }

    /// 校验配置与 string 元素（空串元素合法，表示回退到配置中的 device_key）
    async fn preflight(&self, config: &Value, recipients: &[Value]) -> Result<(), String> {
        serde_json::from_value::<BarkConfig>(config.clone())
            .map_err(|e| format!("Invalid Bark config: {}", e))?;
        string_elements(recipients, "Bark")?;
        Ok(())
    }

    async fn send(
        &self,
        config: &Value,
        recipients: &[Value],
        payloads: &[Value],
    ) -> Result<(), DispatchError> {
        let bark_config: BarkConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid Bark config: {}", e)))?;
        let keys = string_elements(recipients, "Bark").map_err(DispatchError::Abort)?;

        let push_url = format!("{}/push", bark_config.server_url.trim_end_matches('/'));
        let mut errors = Vec::new();
        for key in keys {
            let device_key = if !key.is_empty() {
                Some(key.to_string())
            } else {
                bark_config.device_key.clone()
            };

            let mut push_payload = payloads[0].clone();
            if let Some(ref k) = device_key {
                push_payload["device_key"] = json!(k);
            }

            let url = push_url.clone();
            let result =
                run_http_call("Bark", move || http_client::post_json(&url, &[], &[], &push_payload))
                    .await;
            if let Err(e) = result {
                errors.push(format!("device_key '{}': {}", key, e.into_msg()));
            }
        }
        collect_failures(errors)
    }
}
```

- [ ] **Step 6: 重写 gotify/service.rs**

```rust
use async_trait::async_trait;
use serde_json::{Value, json};
use share_lib::infrastructure::http_client;

use crate::model::channel_config::GotifyConfig;
use crate::services::channel::{
    Channel, DispatchError, collect_failures, run_http_call, string_elements,
};

/// Gotify 推送渠道实现
pub struct GotifyChannel;

impl GotifyChannel {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Channel for GotifyChannel {
    fn channel_type(&self) -> &'static str {
        "gotify"
    }

    /// 校验配置与 string 元素（空串元素合法，表示回退到配置中的 app_token）
    async fn preflight(&self, config: &Value, recipients: &[Value]) -> Result<(), String> {
        serde_json::from_value::<GotifyConfig>(config.clone())
            .map_err(|e| format!("Invalid Gotify config: {}", e))?;
        string_elements(recipients, "Gotify")?;
        Ok(())
    }

    async fn send(
        &self,
        config: &Value,
        recipients: &[Value],
        payloads: &[Value],
    ) -> Result<(), DispatchError> {
        let gotify_config: GotifyConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid Gotify config: {}", e)))?;
        let tokens = string_elements(recipients, "Gotify").map_err(DispatchError::Abort)?;

        let payload = &payloads[0];
        let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let message = payload
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let priority = payload
            .get("priority")
            .and_then(|v| v.as_u64())
            .unwrap_or(5);

        let mut form_body = format!("title={}&message={}&priority={}", title, message, priority);
        if let Some(url) = payload.get("url").and_then(|v| v.as_str()) {
            if !url.is_empty() {
                let extras = json!({
                    "client::notification": {
                        "click": { "url": url }
                    }
                });
                form_body.push_str(&format!("&extras={}", extras));
            }
        }

        let base_url = gotify_config.server_url.trim_end_matches('/').to_string();
        let mut errors = Vec::new();
        for token in tokens {
            let app_token = if !token.is_empty() {
                token.to_string()
            } else {
                gotify_config.app_token.clone()
            };
            let push_url = format!("{}/message?token={}", base_url, app_token);
            let body = form_body.clone();
            let result = run_http_call("Gotify", move || {
                http_client::post_raw(
                    &push_url,
                    &[(
                        "Content-Type".to_string(),
                        "application/x-www-form-urlencoded".to_string(),
                    )],
                    &[],
                    &body,
                )
            })
            .await;
            if let Err(e) = result {
                errors.push(format!("app_token '{}': {}", token, e.into_msg()));
            }
        }
        collect_failures(errors)
    }
}
```

- [ ] **Step 7: 重写 webhook/service.rs**

删除 `dispatch_webhook`（teams_hook 不再复用它；URL 覆盖语义改为逐元素处理）：

```rust
use async_trait::async_trait;
use serde_json::Value;
use share_lib::infrastructure::http_client;

use crate::model::channel_config::WebhookConfig;
use crate::services::channel::{
    Channel, DispatchError, collect_failures, run_http_call, string_elements,
};

/// 通用 Webhook 推送渠道实现
pub struct WebhookChannel;

impl WebhookChannel {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Channel for WebhookChannel {
    fn channel_type(&self) -> &'static str {
        "webhook"
    }

    /// 校验配置与 string 元素（空串元素合法，表示回退到配置中的 webhook_url）
    async fn preflight(&self, config: &Value, recipients: &[Value]) -> Result<(), String> {
        serde_json::from_value::<WebhookConfig>(config.clone())
            .map_err(|e| format!("Invalid Webhook config: {}", e))?;
        string_elements(recipients, "Webhook")?;
        Ok(())
    }

    /// 渲染后的 payload 原样 POST 到每个 URL 元素；空串元素回退用配置中的 webhook_url
    async fn send(
        &self,
        config: &Value,
        recipients: &[Value],
        payloads: &[Value],
    ) -> Result<(), DispatchError> {
        let webhook_config: WebhookConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid Webhook config: {}", e)))?;
        let urls = string_elements(recipients, "Webhook").map_err(DispatchError::Abort)?;

        let mut errors = Vec::new();
        for url in urls {
            let webhook_url = if !url.is_empty() {
                url.to_string()
            } else {
                webhook_config.webhook_url.clone()
            };
            let webhook_payload = payloads[0].clone();
            let result = run_http_call("Webhook", move || {
                http_client::post_json(&webhook_url, &[], &[], &webhook_payload)
            })
            .await;
            if let Err(e) = result {
                errors.push(format!("url '{}': {}", url, e.into_msg()));
            }
        }
        collect_failures(errors)
    }
}
```

- [ ] **Step 8: 重写 teams_hook/service.rs（本改造的核心渠道）**

```rust
use async_trait::async_trait;
use serde_json::{Map, Value};
use share_lib::infrastructure::http_client;

use crate::model::channel_config::WebhookConfig;
use crate::services::channel::{
    Channel, DispatchError, collect_failures, run_http_call,
};
use crate::services::template_render;

/// Teams Hook 推送渠道实现
/// 是通用 webhook 的一层封装：recipient 元素为对象，
/// 每个元素的字段（user/group_id/team_id/channel_id）merge 进模板 variables，
/// 逐元素渲染出定制 payload 后 POST 到配置中的 webhook_url
pub struct TeamsHookChannel;

/// recipient 元素对象允许的 key
const ALLOWED_KEYS: &[&str] = &["user", "group_id", "team_id", "channel_id"];

impl TeamsHookChannel {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Channel for TeamsHookChannel {
    fn channel_type(&self) -> &'static str {
        "teams_hook"
    }

    /// 校验配置与元素 schema：对象、key 合法且值为 string、至少出现一个允许的 key
    async fn preflight(&self, config: &Value, recipients: &[Value]) -> Result<(), String> {
        serde_json::from_value::<WebhookConfig>(config.clone())
            .map_err(|e| format!("Invalid TeamsHook config: {}", e))?;
        for v in recipients {
            let obj = v
                .as_object()
                .ok_or_else(|| "TeamsHook recipient elements must be objects".to_string())?;
            if obj.is_empty() {
                return Err(
                    "TeamsHook recipient element must contain at least one of user/group_id/team_id/channel_id"
                        .to_string(),
                );
            }
            for (k, val) in obj {
                if !ALLOWED_KEYS.contains(&k.as_str()) {
                    return Err(format!("Unknown TeamsHook recipient key '{}'", k));
                }
                if !val.is_string() {
                    return Err(format!("TeamsHook recipient key '{}' must be a string", k));
                }
            }
        }
        Ok(())
    }

    /// 逐元素 merge variables 后渲染；payloads[i] 与 recipients[i] 一一对应
    fn render(
        &self,
        recipients: &[Value],
        template_json: &Value,
        variables: &Map<String, Value>,
    ) -> Result<Vec<Value>, DispatchError> {
        let mut payloads = Vec::with_capacity(recipients.len());
        for v in recipients {
            let obj = v.as_object().ok_or_else(|| {
                DispatchError::Abort("TeamsHook recipient elements must be objects".to_string())
            })?;
            let mut merged = variables.clone();
            for (k, val) in obj {
                merged.insert(k.clone(), val.clone());
            }
            payloads.push(template_render::render_channel(template_json, &merged));
        }
        Ok(payloads)
    }

    /// 逐元素 POST 到配置中的 webhook_url
    async fn send(
        &self,
        config: &Value,
        _recipients: &[Value],
        payloads: &[Value],
    ) -> Result<(), DispatchError> {
        let hook_config: WebhookConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid TeamsHook config: {}", e)))?;

        let mut errors = Vec::new();
        for (i, payload) in payloads.iter().enumerate() {
            let url = hook_config.webhook_url.clone();
            let body = payload.clone();
            let result =
                run_http_call("TeamsHook", move || http_client::post_json(&url, &[], &[], &body))
                    .await;
            if let Err(e) = result {
                errors.push(format!("element {}: {}", i, e.into_msg()));
            }
        }
        collect_failures(errors)
    }
}
```

- [ ] **Step 9: 重写 teams/service.rs（仅签名适配，MessageCard 行为不变）**

imports 改为 `use crate::services::channel::{Channel, DispatchError, collect_failures, run_http_call, string_elements};`，trait impl 改为：

```rust
#[async_trait]
impl Channel for TeamsChannel {
    fn channel_type(&self) -> &'static str {
        "teams"
    }

    /// 校验配置与 string 元素（空串元素合法，表示回退到配置中的 webhook_url）
    async fn preflight(&self, config: &Value, recipients: &[Value]) -> Result<(), String> {
        serde_json::from_value::<TeamsConfig>(config.clone())
            .map_err(|e| format!("Invalid Teams config: {}", e))?;
        string_elements(recipients, "Teams")?;
        Ok(())
    }

    async fn send(
        &self,
        config: &Value,
        recipients: &[Value],
        payloads: &[Value],
    ) -> Result<(), DispatchError> {
        let teams_config: TeamsConfig = serde_json::from_value(config.clone())
            .map_err(|e| DispatchError::Abort(format!("Invalid Teams config: {}", e)))?;
        let urls = string_elements(recipients, "Teams").map_err(DispatchError::Abort)?;

        // Teams 固定使用 MessageCard 格式（构建逻辑与原实现一致，基于 payloads[0]）
        let payload = &payloads[0];
        // ... 原有 MessageCard 构建代码逐行保留，得到 teams_payload ...

        let mut errors = Vec::new();
        for url in urls {
            let webhook_url = if !url.is_empty() {
                url.to_string()
            } else {
                teams_config.webhook_url.clone()
            };
            let body = teams_payload.clone();
            let result = run_http_call("Teams", move || {
                http_client::post_json(&webhook_url, &[], &[], &body)
            })
            .await;
            if let Err(e) = result {
                errors.push(format!("url '{}': {}", url, e.into_msg()));
            }
        }
        collect_failures(errors)
    }
}
```

注意：上段中 `// ... 原有 MessageCard 构建代码 ...` 指把现文件 43-87 行（known_keys 到 potentialAction）原样搬入，仅把开头的 `let payload = payload` 引用改为 `&payloads[0]`。

- [ ] **Step 10: 全量编译检查**

Run: `cargo check -p yell`
预期：通过，无 `dispatch_template` / 旧签名相关错误。若有遗漏的调用点（如 manage_service 中的模板预览仍用 `render_template`，属正常保留），编译器会指出，按新签名适配。

- [ ] **Step 11: Commit**

```bash
git add yell/src/services
git commit -m "refactor(yell): split Channel trait into preflight/render/send, recipient as array"
```

---

### Task 4: alias 写入侧结构校验

**Files:**
- Modify: `yell/src/api/alias_manage.rs:46-53`（create）、`yell/src/api/alias_manage.rs:91-103`（update）

**Interfaces:**
- Consumes: `notify_service::validate_recipients_shape(value: &Value) -> Result<(), String>`（Task 3）
- Produces: 无新接口

- [ ] **Step 1: create_alias 加校验**

imports 增加 `services::notify_service`（现有 `use crate::{model::..., services::manage_service};` 改为 `use crate::{model::notification_alias::{NewNotificationAlias, UpdateNotificationAlias}, services::{manage_service, notify_service}};`）。

在 `let recipients = req.get("recipients").cloned().ok_or_else(...)?;` 之后插入：

```rust
    if let Err(msg) = notify_service::validate_recipients_shape(&recipients) {
        return Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(400, "Bad Request", Some(msg), 1),
        ));
    }
```

- [ ] **Step 2: update_alias 加校验**

在构造 `UpdateNotificationAlias` 之前插入：

```rust
    if let Some(recipients) = req.get("recipients") {
        if let Err(msg) = notify_service::validate_recipients_shape(recipients) {
            return Err(MailManErrResponser::mapping_from_mme(
                share_lib::data_structure::MailManErr::new(400, "Bad Request", Some(msg), 1),
            ));
        }
    }
```

- [ ] **Step 3: 验证编译并提交**

Run: `cargo check -p yell`
预期：通过。

```bash
git add yell/src/api/alias_manage.rs
git commit -m "feat(yell): validate recipients shape on alias create/update"
```

---

### Task 5: 文档与示例同步

**Files:**
- Modify: `yell/doc/channels_message_format-CN.md`（约 26-57 行：三元组与 recipient 语义）
- Modify: `yell/doc/api/openapi/openapi.yaml`（recipients / alias 相关 schema，约 660-690、790-830 行附近）
- Modify: `yell/doc/api/http/alias.http`、`yell/doc/api/http/notify.http`（所有 `"recipient": "..."` 改为 `"recipient": ["..."]`）
- Modify: `yell/Readme.md`、`yell/Readme_ZH-CN.md`（渠道表中 recipient 说明）
- Modify: `yell/doc/api/groovy/yell.groovy`（`parseRecipients` 的 doc 注释示例改为数组格式；逻辑为透传无需改）

**Interfaces:**
- Consumes: Task 3 的最终行为
- Produces: 无

- [ ] **Step 1: channels_message_format-CN.md**

要点更新（保持原文风格，逐处修改）：
- `Vec<(channel_type, recipient, instance)>` 的描述改为 `RecipientTarget { channel_type, instance, recipients: Vec<Value> }`；
- 示例 `[{ "channel_type": "bark", "recipient": "aBcDeFg...", "instance": "default" }]` 改为 `"recipient": ["aBcDeFg..."]`；
- "recipient：渠道标识..." 段落补一句：recipient 为非空数组，元素 schema 由各渠道定义（string 或 teams_hook 的对象）；
- `Channel` trait 三方法的描述（`channel.rs:59-82` 引用处）改为 `channel_type` / `preflight` / `render` / `send`；
- smtp 章节删除 `,`/`;` 分隔符多地址的描述，改为"数组元素即多个 To"；
- teams_hook 章节描述对象元素 `{user, group_id, team_id, channel_id}` 的 merge 渲染语义。

- [ ] **Step 2: openapi.yaml**

recipients 元素的 `recipient` schema 由 `type: string` 改为：

```yaml
                recipient:
                  type: array
                  minItems: 1
                  items: {}
                  description: 接收目标数组；元素类型按渠道定（string 或 teams_hook 对象）
```

alias 相关示例同步加方括号。

- [ ] **Step 3: http 示例**

`alias.http` / `notify.http` 中全部 `"recipient": "xxx"` 改为 `"recipient": ["xxx"]`（smtp 多地址示例改为 `["user1@example.com", "user2@example.com"]`，删除 `, `/`; ` 分隔写法示例）。

- [ ] **Step 4: Readme 双版本 + groovy 注释**

Readme 渠道表的 recipient 列说明同步为数组；groovy `parseRecipients` 注释中的示例改为数组格式。

- [ ] **Step 5: Commit**

```bash
git add yell/doc yell/Readme.md yell/Readme_ZH-CN.md
git commit -m "docs(yell): update recipient array format across docs and examples"
```

---

### Task 6: 收尾质量门

**Files:** 无（纯验证）

- [ ] **Step 1: 格式化检查**

Run: `build/fmt_all_ws.sh -- --check`
预期：通过；不通过则 `cargo fmt --all` 后把 yell 相关格式改动并入 Task 3/4 的修正 commit（或单独 `style:` commit）。

- [ ] **Step 2: Clippy**

Run: `cargo clippy -p yell --all-targets`
预期：无新增 warning（本项目 clippy 仅记录不阻断，但本次改动不得引入新警告）。

- [ ] **Step 3: workspace 级检查**

Run: `cargo check --workspace`
预期：通过（确认未误伤其他 crate——理论上改动都在 yell 内）。

- [ ] **Step 4: migration 再验证**

```bash
cd yell && diesel migration redo --database-url "postgres://be:Virtuos%40BE-SHA@10.72.2.12:5432/yell_test"
```

预期：down + up 均成功，库内 recipient 全为数组。

---

## Self-Review 记录

- Spec 覆盖：§1 数据格式 → Task 3 Step 3/8 + Task 4；§2 migration → Task 1；§3 trait → Task 3 Step 1；§4 各渠道 → Task 3 Step 4-9；§5 渲染路由 → Task 2 + Task 3 Step 2；§6 错误语义 → Task 3 各渠道 send/preflight；§7 文档 → Task 5；§8 验证 → Task 6。
- 无单测步骤：遵项目规约（Global Constraints 第 1 条），以编译/migration/手工清单替代。
- 类型一致性：`RecipientTarget` / `string_elements` / `collect_failures` / `into_msg` / `render_channel` / `get_channel_json` / `validate_recipients_shape` 在产出与消费处签名一致。
