# 改进待办（Improvement TODO）

> 来源：`analysis.md` 三层结构审查 + 工程规范化遗留项
> 创建：2026-08-26 ｜ 状态：初版，待补充
> 用法：每条完成后再 `[x]` 勾掉；优先级 P0（先做/共性）> P1 > P2

## P0 —— 共性模式收敛（一次改动，多处受益）

- [x] **收敛 `refresh_master` 集体越层**：cmdb / cloud-api / jc-commander / file-agent 四个 crate
  的 `api/system_manage.rs` 在 handler 里直接做两次 ureq 外呼。方案：在 share-lib 新增
  "子系统自注册/心跳" 公共模块（web feature 下），各 crate 调用同一实现
- [x] **确立"非 DB 原子操作"的归属层**：目前 HTTP(ureq) / MQ(lapin) / 进程派生(Command)
  散落各处。方案：统一约定 `infrastructure/`（或沿用 `util/`）作为非 DB 原子操作层，
  封装 `http_client`、`mq_client`、`process_runner` 三个原子模块骨架，写入骨架规范
- [x] **HTTP 外呼原子模块**：抽离 ureq 调用（URL/header/body/响应解析），
  首批迁移 watchman 的 `subsys_service.rs:295-307`、`webhook_service.rs:163-218`
- [x] **编排层函数统一 async 化**（编码规范已立规，见 `doc/coding_rules-*.md`
  「编排层函数统一为 async」）：存量同步 service 函数全量迁移为 `async fn`，
  handler 调用点改 `.await`。范围：6 个 web crate 的 `service/`、yell 的
  `NotificationRouter` 方法、jc-worker 的 `task`/`json_rpc`；model 层不在范围内。
  建议按 crate 分批进行，每批同步更新全部调用方

## P1 —— 分层结构整改（按 crate）

### jc-commander（当前最不满足）

- [x] `call_sync`/`call_async` 编排从 handler 下沉到 service 层
  （api/script_caller.rs:20-161），handler 只留清洗
- [x] MQ 投递收敛：`create_channel`/`basic_publish` 封装为原子模块，
  消除 `{prefix}_async` 队列名两处拼装（script_caller.rs:176 vs scheduler.rs:227-234）
- [x] `cron_job::new` 的实体组装（uuid 生成、默认值、双表实体）移出 handler
  （api/cron_job.rs:36-126）；顺带评估双表写入是否需要事务
- [x] `util/scheduler.rs` 越过 service 直持 db_pool/mq_pool 的问题：
  明确 scheduler 在分层中的位置（建议视为独立调度器组件，但其 DB/MQ 调用走原子模块）

### jc-worker

- [x] 建立清洗层：MQ payload 定义强类型输入结构 + 反序列化校验，
  杜绝链式 `unwrap`/`expect` 导致脏消息在 ack 前 panic（service/task.rs:10-36）
- [x] 进程执行收敛为原子模块（替代 task.rs:47-51 的内联 Command）
- [x] MQ 连接/消费/ack 从 main.rs 抽为独立模块（main.rs:42-198）
- [x] `json_rpc.rs` 更名/归位：它是 HTTP 外呼原子操作，不应叫 service

### file-agent

- [x] upload 的 multipart 解析 + 落盘 IO 从 handler 移出
  （api/file_manage.rs:68-146），handler 只留校验
- [x] 文件 IO 收敛为原子层（替代 service 里的直接 `std::fs` 调用）；
  顺带处理名不副实的 `util/file_op.rs`（无 IO、无人消费的烂尾 token 结构，删除或完成）
- [x] handler 取参改为校验式清洗，替换裸 `unwrap`

### yell

- [x] 9 个管理端点补编排层：handler → service → model，
  消除 handler 直接 `web::block` 调 model（api/notify.rs:278-799）
- [x] `resolve_recipients` 的 alias 查库从 handler 移到编排层（api/notify.rs:34-42）
- [x] router 直接查库（ChannelConfig/模板）改为由编排层注入或经 service 获取
  （notification_router.rs:233-260, 138-146）
- [x] 渠道 [查配置→建记录→外呼→更新记录] 小编排在 5 渠道 × 2 方法中重复，
  收敛为 channel.rs 中的统一流程骨架，渠道只实现"外呼"本身
- [x] 模板 render 业务逻辑从 model 移出（model/notification_template.rs:185-237）
- [x] 处理 wecom 空目录死代码（实现或删除）

### cloud-api

- [x] script 链路的子进程派生/目录列举收敛为原子模块（service/script_caller.rs:62-68, :114）
- [x] script handler 伪清洗（`req["api_name"].as_str().unwrap()`）改为校验式清洗
- [x] AK/SK 明文上命令行的安全隐患评估（service/script_caller.rs 子进程 argv 传凭据）

### watchman-backend

- [x] `subsystem_call` 链路补清洗层：消除 service 里 `subsys_params["target"].as_str().unwrap()`
  （subsys_service.rs:298-300）
- [x] 删除 legacy `Authentication` 中间件死代码（middleware/auth_middleware.rs:52-246），
  并评估中间件内嵌编排（:487-506）与 `get_me` 查询序列的去重
- [x] 删除 `subsys_service.rs:38-80` 注释掉的死代码

### cmdb-backend

- [x] handler 裸 `unwrap` 取 `data.get("table")` 改为 400 错误返回
  （table_manage.rs:19,161,489,824）
- [x] 删除空文件 `api/db_manage.rs`
- [x] 评估编排层空心化：18 个同构透传 service 是否保留（作为错误码翻译层有其价值，
  需在骨架规范中明确这一层的定位）

## P1 —— 清洗层统一

- [x] **GET filter 穿透问题**：所有 crate 的 GET 链路把原始 Query map 穿透三层直达 model
  拼动态查询。方案：清洗层定义各表允许的 filter key 白名单 + 类型，model 只收强类型 filter
- [x] **`from_map` 归位**：清洗逻辑目前物理位置在 model 文件里（全仓库统一惯例），
  评估独立为清洗层模块（dto/ 或 api 层内部），并同步命名规范文档
- [x] cloud-api / file-agent / yell 的 POST 链路接入 `from_map` 式清洗惯例

## P2 —— 疑似 bug 修复（均来自 analysis.md，修复前先确认）

- [x] watchman signup 清洗把 `username` 赋给 `passwd` 字段（api/account_manage.rs:69-76）
- [x] cmdb `from_map` 读取 `"plantform"` 拼写错误，platform 字段永远为 None
  （model/cloudserver/instance.rs:68）
- [x] jc-commander `times` 字段误读 `map.get("frequency")`（model/cron_job.rs:53）
- [x] jc-commander `call_async` 的 exec_type 硬编码为 `"sync"`（api/script_caller.rs:186）
- [x] jc-worker `output` 二次 unwrap，stdout/stderr 皆空时 panic（service/task.rs:53-94）
- [x] jc-commander cron 双表写入无事务（service/cron_job.rs:37-71）

## P2 —— 工程遗留

- [ ] 在有 OpenSSL 的环境跑一次 `build/check_all_ws.sh` + `build/clippy_all_ws.sh`，
  验证 5 个 diesel crate（本机 Windows 缺 OpenSSL 未验证）
  （**GitHub Actions `check.yml` 已建，首次运行即完成此验证**）
- [x] yell 补登记进 `docker-compose.yaml`（端口 9005）
- [x] yell 补双语 README（`Readme.md` + `Readme_ZH-CN.md`）
- [x] yell 迁移 `2026-05-22-010000_add_webhook_presets` 补 down.sql
- [x] 迁移自动化评估：`diesel_migrations` 依赖声明了但全仓库未使用，
  决定 embed_migrations 自动执行还是继续手工 CLI（并写入骨架规范）
- [x] CI 增强：`.gitea/workflows/build.yaml` 增加 `cargo fmt --check` / `clippy` 步骤
- [x] GitHub Actions 自动 `cargo check`：远程仓库托管在 GitHub，但 CI 目前只有
  Gitea Actions。新建 `.github/workflows/check.yml`：push/PR 触发，`ubuntu-latest`
  runner 上跑 `cargo check --workspace`（runner 需预装 libpq/openssl 开发包，
  pq-sys bundled 依赖链需要；可顺带把 `cargo fmt --all -- --check` 一并纳入），
  与 Gitea CI 的关系（并存还是迁移）一并决策（决策：并存，Gitea 负责构建发布、
  GitHub 负责检查门禁）
- [ ] clippy 基线收敛后开启 `-D warnings`
- [x] file-agent 清理无使用方的残留依赖（实际仅 `crossbeam` 无使用已移除；
  `uuid` 在 config/server.rs 生成 subsys_uuid 仍在使用，保留）

## P2 —— 规范文档同步

- [x] 分层整改完成后，回写 `doc/project_skeleton-*.md`：补充非 DB 原子操作层、
  清洗层定位、scheduler/middleware 等横切组件的职责约定
- [x] `analysis.md` 中"现状与例外"类内容随整改进度更新

---

## 评估记录

### cloud-api AK/SK 明文上命令行（2026-08-26 评估，本期只记录不改动）

**现状**：`service/script_caller.rs` 派生 python 子进程时，把云账号 AK/SK 作为
argv 传入（`<python> <script>.py <AK> <SK> <region> <params>`）；`script/` 下全部
15 个业务脚本以 `sys.argv[1]` / `sys.argv[2]` 读取凭据，无 env/stdin 读取实现。

**风险描述**：子进程 argv 对同机所有进程可见（`ps -ef`、`/proc/<pid>/cmdline`），
运行 cloud-api 的主机上任何非特权用户都可在脚本运行窗口内读到云账号凭据；
进程崩溃时的 core dump、运维采集工具也可能把 argv 带出主机。

**影响面**：只影响部署主机本地的信任边界（同机其他用户/进程），不影响网络传输；
cloud-api 的调用方与响应协议均不涉及该问题。整改需 Rust 编排层与全部 python
脚本同步改动，属于两侧协议变更。

**建议方案**（后续实施时）：
- python 脚本统一改为从环境变量（如 `CLOUD_API_AK` / `CLOUD_API_SK`）或 stdin
  读取凭据；环境变量在 Linux 上同样可见于 `/proc/<pid>/environ`，但权限约束更严
  （仅属主可读，需 `ptrace` 权限），stdin 则完全不落进程列表，二者均优于 argv；
- Rust 侧配合：`process_runner::run` 目前不支持传 env/stdin，需扩展该原子操作
  （或在本 crate 内用 `Command.env()` / 管道 stdin 派生）；
- 15 个脚本 + 每个产品目录的 `package_import.py` 入口约定需一并修改，
  变更面明确但覆盖整个 script/ 目录。

**当前决策**：本期只记录、不改协议。原因：① 属两侧协议变更，超出本期
"不改变对外行为"的整改范围；② 风险等级受部署形态约束——若 cloud-api 独占
主机/容器运行，同机无不可信进程，实际暴露面有限；③ `process_runner` 尚无
env/stdin 能力，改造应先落 share-lib 基础设施再推进。

---

## 补充区（维护者手写）
