# Yell

yell 是 Kernel Master 项目的通知服务.
它监听 **9005** 端口, 向多种渠道 (Bark / Gotify / Mail(SMTP) / Teams / Teams Hook / Webhook)
分发通知, 支持模板渲染、渠道多实例配置、收件人别名, 并将投递记录持久化到 PostgreSQL.

## 代码规则

- 所有 ORM 相关的操作放置在 ***model*** 目录下的相应模块中.

- 渠道发送的编排逻辑放置在 ***service*** 目录下, 渠道外呼实现放置在 ***infra*** 目录下.

    每个渠道 (`bark/`、`gotify/`、`mail/`、`teams/`、`teams_hook/`、`webhook/`) 实现
    `service/channel.rs` 中定义的 `Channel` trait (`channel_type` / `preflight` /
    `render` / `send`). `service/notification_router.rs`
    负责把请求路由到指定的渠道实例并记录投递结果.

- 与 API 响应相关的逻辑放置在 ***api*** 目录下的相应模块中.

    输入 map 通过 `filter.rs` 解析 (`from map`), 未传入的值处理为 `None`.
    所有错误通过 MailMan 体系返回 (`MailManErr` → `MailManErrResponser`).

## APIs

所有 API 以 `/api` 范围开头. 除 `/hey` 与 `/manage` 外, 所有 scope 都包裹了 `Authentication` 中间件.

所有成功响应均以 `MailManOk` (`{code, key, data}`) 包装返回, 包括查询类 GET
端点 (`/template/get`、`/template/help`、`/record/get`、`/channel/get`、`/alias/get`),
其载荷位于 `data` 字段.

### /hey

| 资源  | 支持的方法 | 功能     | 备注               |
| :---: | :--------: | :------- | :----------------- |
|   /   | `GET` `POST` | 健康检查 | 公开接口, 无需认证 |

### /manage

|      资源       | 支持的方法 | 功能                        | 备注                                          |
| :-------------: | :--------: | :-------------------------- | :-------------------------------------------- |
| /refresh_master |   `POST`   | 向 watchman 刷新本实例注册 | 公开接口, 无需认证; 实现于 `api/system_manage.rs` |

### /notify

|   资源    | 支持的方法 | 功能           | 备注                                                                                  |
| :-------: | :--------: | :------------- | :------------------------------------------------------------------------------------ |
| /template |   `POST`   | 使用模板发送   | 需要 `template_name` 与 `variables`; `recipients` 只接受别名 (alias) 字符串, 经 `notification_aliases` 表解析为实际收件人 |

原直接发送端点 `POST /api/notify/send` 已删除, 模板发送是唯一的发送路径.

### /template

|  资源   | 支持的方法 | 功能         | 备注                                                              |
| :-----: | :--------: | :----------- | :---------------------------------------------------------------- |
|   /get  |   `GET`    | 获取模板列表 | 支持 query 过滤; `is_enabled` 传 `"true"`/`"false"` 字符串即可 |
|  /help  | `GET` `POST` | 获取指定模板的模拟渲染示例 | 见下文 [模板帮助](#模板帮助) |
|   /new  |   `POST`   | 创建模板     | 分渠道模板体: `smtp` / `bark` / `gotify` / `teams_hook` / `webhook` |
| /update |   `POST`   | 更新模板     | 需要 `id`                                                         |
| /delete |   `POST`   | 删除模板     | 需要 `id`                                                         |

#### 模板帮助

`GET /api/template/help?name=<模板名>` 或
`POST /api/template/help`, JSON body 为 `{"name": "<模板名>"}`.

返回该模板按渠道分组的模拟渲染 JSON 示例 (以 `MailManOk` 包装, 载荷在 `data` 字段). 渲染规则
(实现于 `service/template_render.rs::render_example`):

- 所有 `{{var}}` 占位符统一填充为字符串 `"TEST"`;
- 模板中的数字 / 布尔字面量原样保留;
- 响应中只包含模板实际配置的渠道列.

错误响应: `name` 缺失或非字符串返回 `400`; 模板不存在或已禁用返回 `404`.

示例 — `POST /api/template/help`, body 为 `{"name": "webhook_json"}`
(迁移预置模板, 仅配置了 `webhook` 渠道):

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

| 资源  | 支持的方法 | 功能             | 备注           |
| :---: | :--------: | :--------------- | :------------- |
|  /get |   `GET`    | 查询通知投递记录 | 支持 query 过滤 |

### /channel

|  资源   | 支持的方法 | 功能             | 备注                                                  |
| :-----: | :--------: | :--------------- | :---------------------------------------------------- |
|   /get  |   `GET`    | 获取渠道配置列表 |                                                       |
| /update |   `POST`   | 更新渠道实例配置 | 需要 `channel_type`、`name` (实例名) 和 `config_json` |

### /alias

|  资源   | 支持的方法 | 功能         | 备注                        |
| :-----: | :--------: | :----------- | :-------------------------- |
|   /get  |   `GET`    | 获取所有别名 |                             |
|   /new  |   `POST`   | 创建别名     | 需要 `name` 和 `recipients`; 每个元素的 `recipient` 必须是非空数组 (元素类型按渠道定: string, `teams_hook` 为对象) |
| /update |   `POST`   | 更新别名     | 需要 `id`                   |
| /delete |   `POST`   | 删除别名     | 需要 `id`                   |

## 支持的渠道

|   渠道    | `channel_type` | 备注                                                                     |
| :-------: | :------------: | :----------------------------------------------------------------------- |
|   Bark    |     `bark`     | 通过 Bark 服务器推送 iOS 通知                                            |
|  Gotify   |    `gotify`    | 通过自托管 Gotify 服务器推送                                             |
|   Mail    |     `smtp`     | 通过 SMTP 发送邮件 (lettre)                                              |
|   Teams   |    `teams`     | 通过 Incoming Webhook 发送 MessageCard; 仍在 router 注册, 但模板已无 `teams` 列 (改名为 `teams_hook`), 模板发送路径下实际不可达 |
| Teams Hook |  `teams_hook`  | 通过 Incoming Webhook 发送到 Microsoft Teams; recipient 元素为对象 (`user` / `group_id` / `team_id` / `channel_id`, 至少一个), 其字段 merge 进模板 variables 后逐元素渲染, POST 到实例配置的 webhook URL |
|  Webhook  |    `webhook`   | 通用 HTTP webhook; 渲染后的模板 payload 原样作为请求体发送                |

每种渠道类型支持多实例配置 (按 `channel_type` + 实例 `name` 区分).

## 部署 和 依赖

- 依赖库

    - openssl
    - libpq

- 依赖服务

    - PostgreSQL

### 配置文件

将 `yell_backend.template.toml` 复制为工作目录下的 `yell_backend.toml` 并修改.
主要配置项:

```toml
[server_config]
# 日志配置
log_path = "logs/yell.log"
log_level = "INFO"
clear_log = "True"
# 服务参数
listen_addr = "127.0.0.1"
listen_port = 9005
workers = 1
# 实例注册信息 (上报给 watchman)
uuid = "550e8400-e29b-41d4-a716-446655440000"
register_name = "yell"
master_addr = "127.0.0.1"
master_port = 8000
# CORS 与认证白名单
allowed_origin_list = ["http://localhost:3000", "http://127.0.0.1:3000"]
authenticate_bypass = ["/api/refresh_master"]

# 数据库配置
[db_config]
db_str = "postgres://postgres:password@localhost:5432/yell"
```

### 数据库迁移

迁移文件位于 `migrations/` 目录, 使用 diesel CLI 手工执行:

```sh
diesel migration run
```

### 运行

```sh
# 在 workspace 根目录构建
cargo build --release -p yell
# 确保工作目录下存在 yell_backend.toml 后运行
./target/release/yell
```

或通过 docker-compose (workspace 根目录): `docker-compose --profile run up -d yell`.
