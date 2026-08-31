# yell alias recipient 数组化改造 — 设计文档

日期：2026-08-31
状态：已确认（用户逐节审批通过）
范围：仅 `yell` crate

## 背景与目标

`notification_aliases.recipients`（JSONB 数组）中每个元素的 `recipient` 字段目前是纯 string，
各渠道对这一个 string 的解释各不相同：smtp 用 `,`/`;` 分隔符 hack 多地址，bark 是单个
device_key，gotify 是单个 app_token，webhook/teams_hook 是单个 URL。

目标：把 `recipient` 改为数组类型，元素 schema 由各渠道自行定义和校验，
以支撑 teams_hook 的"按接收人注入模板变量"需求：

```json
{"channel_type": "teams_hook", "instance": "teams-main", "recipient": [
  {"user": "chuang.shen", "team_id": "XXX", "group_id": "XXX", "channel_id": "XXX"}
]}
```

teams_hook 的每个元素对象会 merge 进模板 variables，逐元素渲染出定制 payload 后
POST 到 teams webhook（接收端靠 `user`/`team_id`/`channel_id` 做 @mention / 频道路由）。

## 关键决策（brainstorming 结论）

| 决策点 | 结论 | 理由 |
|---|---|---|
| 数组语义 | 渠道内聚合（非 fan-out） | 避免 smtp 扇出产生过多邮件；一条三元组只写一条发送记录 |
| 元素类型 | `array[any]`，渠道自校验 | teams_hook 需要对象元素；smtp 等仍只接受 string |
| 兼容策略 | 硬切换 + 数据迁移 | 格式唯一、代码最干净；migration 负责改写存量 string |
| 渲染位置 | 渲染下沉到所有渠道（方案 1） | 保证整体代码库结构统一，接受较大改动 |

## 1. 数据格式

alias `recipients` 元素结构：

```json
{"channel_type": "bark", "instance": "bark-main", "recipient": ["key1", "key2"]}
{"channel_type": "teams_hook", "instance": "teams-main", "recipient": [
  {"user": "chuang.shen", "team_id": "X", "group_id": "Y", "channel_id": "Z"}
]}
```

- `recipient` 必须为**非空数组**；string 一律拒绝（400，报清晰错误信息）。
- 元素 schema 按渠道定义：
  - smtp / bark / gotify / webhook：只接受 string 元素；
  - teams_hook：只接受对象元素，四个 key（`user` / `group_id` / `team_id` / `channel_id`）
    均为可选 string，但至少出现一个。
- 编排层（`notify_service.rs`）只校验"数组非空"，元素 schema 由渠道在 preflight 校验。

## 2. Migration

新增 `yell/migrations/2026-08-31-000001_recipient_string_to_array/`：

- `up.sql`：
  1. 用 `jsonb_array_elements ... WITH ORDINALITY` + `jsonb_set` + `jsonb_agg`
     把每个元素的 `recipient` string 改写为单元素数组（保序）；
  2. `notification_records.recipient` 从 `VARCHAR(512)` 改为 `TEXT`
     （数组 JSON 序列化后 512 字符放不下多目标场景）。
- `down.sql`：
  1. 单元素数组还原为 string；多元素数组取第一个元素（**有损，注释中明确标注**）；
  2. `recipient` 列改回 `VARCHAR(512)`。

## 3. Trait 重构（channel.rs）

`Channel` trait 从单一 `dispatch_template` 拆为三步，所有渠道同构：

- `preflight(config, recipients)` — 校验配置 + 元素 schema；不合法返回 `DispatchError::Abort`
  （不写记录，错误直接上抛）；
- `render(recipients, template_json, variables) -> Result<Vec<Value>, _>` —
  普通渠道恒返回 1 个 payload；teams_hook 逐元素 merge variables 后渲染，返回 N 个，
  `payloads[i]` 与 `recipients[i]` 按下标一一对应；
- `send(config, recipients, payloads)` — 执行外呼。元素间互不阻断，全部尝试完毕后，
  有错才返回 `DispatchError::Failed`（错误信息汇总并标注失败元素）。

`deliver_template` 骨架变为：

```
取配置 → preflight → render → 用 payloads[0] 创建 pending 记录
       → send → finalize（sent/failed 更新记录）
```

发送记录 `recipient` 列存元素数组的紧凑 JSON（`serde_json::to_string(recipients)`）。

## 4. 各渠道行为

- **smtp**：全部 string 元素作为一封邮件的多个 To（地址格式校验在 preflight 完成，
  非法地址 `Abort`）；现有 `,`/`;` 分隔符解析逻辑删除；
- **bark / gotify**：渲染一次，逐 device_key / app_token POST；
- **webhook**：渲染一次，逐 URL POST；空字符串元素回退用配置里的 `webhook_url`
  （沿用现有"非空覆盖"语义）；
- **teams_hook**：逐元素 merge `variables`（元素字段注入/覆盖）→ 渲染 →
  POST 到配置的 `webhook_url`；普通 webhook 不做 merge，直接转发 vars 渲染结果；
- **teams**（旧渠道）：模板无 `teams` 列，三元组照旧被跳过，仅做 trait 签名适配。

## 5. 渲染与路由

- `template_render.rs` 暴露单渠道渲染函数 `render_channel(template_column_json, variables)`；
  原整模板 `render_template` 保留给模板预览类 API；
- `notification_router.rs` 不再预渲染，按三元组的 `channel_type` 取模板对应 JSONB 列
  （无对应列的三元组照旧跳过）+ variables 透传给 `deliver_template`。

## 6. 错误语义

- 元素 schema 不合法 → preflight `Abort`，不写记录，错误随本次请求的结果返回；
- 部分元素发送失败 → 记录标记 `failed`，`error_msg` 汇总并标注失败元素；
  已成功的元素不重试；
- smtp 单邮件多 To 为原子操作，失败即整体 `failed`。

## 7. 影响面

代码：

- 新增 migration（`up.sql` / `down.sql`）；
- `yell/src/model/schema.rs`：`notification_records.recipient` Varchar → Text；
- `yell/src/services/notify_service.rs`：三元组改 struct（`channel_type` / `instance` /
  `recipients: Vec<Value>`），数组非空校验；
- `yell/src/services/channel.rs`：trait 三步拆分、`deliver_template` 重写、记录辅助函数适配；
- `yell/src/services/notification_router.rs`：去掉预渲染，透传模板列 + variables；
- `yell/src/services/template_render.rs`：新增 `render_channel`；
- 6 个渠道实现（mail / bark / gotify / webhook / teams_hook / teams）按新 trait 重写；
- `yell/src/api/alias_manage.rs`：alias 写入侧校验同步（数组格式）。

文档与调用方：

- `yell/doc/channels_message_format-CN.md`、`yell/doc/api/openapi/openapi.yaml`、
  `yell/doc/api/http/*.http` 示例、`Readme.md` / `Readme_ZH-CN.md`；
- `yell/doc/api/groovy/yell.groovy`（若构造 alias/recipients 结构）。

## 8. 验证

- 遵本期规约不补单测；
- `cargo check --workspace` + `cargo clippy --workspace --all-targets` +
  `build/fmt_all_ws.sh -- --check` 必须通过；
- 用 `yell/doc/api/http/` 示例做手工验证清单：新格式 alias 的创建、
  smtp 多 To、bark 多 key、teams_hook 对象元素 merge 渲染、旧格式 string 被拒（400）。
