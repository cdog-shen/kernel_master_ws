# 子系统三层结构符合性分析

> 分析对象：workspace 内 7 个服务 crate（share-lib 为公共库，不在此列）
> 分析方式：逐 crate 代码审查，证据以 `文件:行号` 标注
> 日期：2026-08-25

## 目标模型定义

1. **清洗/反序列化层**：JSON 进来后只做数据清洗、校验、反序列化，不含业务逻辑
2. **编排层**：只负责调用下层函数、组合流程、汇总数据，不实现原子操作
3. **原子操作层**：DB CRUD / 文件 IO / MQ 收发 / HTTP 外呼等原子操作，只被编排层调用

## 总体结论

| crate | 结论 | 一句话概括 |
|---|---|---|
| watchman-backend | 基本满足 | 三层主链路清晰，HTTP 外呼内联在编排层是主要缺口 |
| cmdb-backend | 基本满足 | 分层最严格，但编排层退化为纯透传（坍缩为两层半） |
| cloud-api | 部分满足 | account_db 链路达标，script 链路编排层混入进程派生 |
| jc-commander | 不满足 | 核心流程的编排+原子操作上溢到 handler，MQ 投递无收敛 |
| jc-worker | 不满足 | 层次坍缩：清洗/编排/进程执行挤在一个函数里 |
| file-agent | 不满足 | 实际是两层扁平结构，service 直接做 fs 原子操作 |
| yell | 部分满足 | 发送链路三层骨架清晰，9 个管理端点跳过编排层直连 model |

## 各子系统详述

### 1. watchman-backend —— 基本满足

标准链路 `api/account_manage.rs → service/account_service.rs → model/*.rs` 执行良好：
handler 用 `from_map()` 清洗、service 只做编排与错误码映射、DB 操作全部收敛在 model。
`login`（service/account_service.rs:27，查用户→签 JWT→更新登录时间→写 token 四步编排）
与 `get_me`（:193，四次 model 调用聚合响应）是合格的编排范例。

**不满足点：**

- HTTP 外呼原子操作（ureq 拼 URL/header/body、读响应）直接内联在编排层：
  `service/subsys_service.rs:295-307`、`service/webhook_service.rs:163-218`（高）
- `subsystem_call` 链路清洗层缺位：原始 Map 从 handler 透传到 service 后
  `subsys_params["target"].as_str().unwrap()` 裸取，缺字段即 panic
  （api/subsys_manage.rs:123-131 → service/subsys_service.rs:298-300）（高）
- GET filter 的 key 解释下沉到原子层：model 直接解析 `Map<String, Value>` 拼动态查询
  （model/user.rs:152-178 及各 model 的 `get_all_with_filter`），清洗层与原子层边界模糊（中）
- 鉴权中间件内嵌编排逻辑（五步 model 调用 + 权限规则，middleware/auth_middleware.rs:487-506），
  与 `account_service::get_me` 的查询序列高度重复；另有已被取代但仍编译保留的
  legacy `Authentication` 中间件（:52-246）（中）

**独有特点：** InputStream/OutputStream/Model 三结构体模式；清洗函数 `from_map` 物理位置在
model 文件里但被 api 层调用；token 先 update 失败回退 insert 的 fallback 编排。

### 2. cmdb-backend —— 基本满足（但编排层空心化）

全仓库分层最严格的 crate：18 个 service 文件 152 个函数全部同构为
"一次 model 调用 + MailMan 包装"，service 内零 diesel/IO/HTTP，无 service 互调。

**不满足点：**

- `refresh_master` 在 handler 内直接做两次 ureq HTTP 外呼 + 响应编排，完全跳过
  service/model（api/system_manage.rs:16-58）（高）
- 编排层退化为纯透传，没有任何"组合多个原子调用"的编排——三层实际坍缩为
  "清洗 → 转发 → 原子"（低，无违规但层存在价值存疑）
- `from_map` 清洗逻辑实现在 model 文件里，model 同时承载反序列化与 DB 两种职责（中）
- handler 用 `unwrap()` 取 `data.get("table")`，缺字段 panic（api/table_manage.rs:19,161,489,824）（低）

**独有特点：** 单入口 + `table` 字段巨型 match 分发（table_manage.rs 单文件 974 行，4 个端点
覆盖 18 张表）；model/service 按子系统双镜像分目录；`full-schema` feature 门控托管其他
子系统的表结构。

### 3. cloud-api —— 部分满足

account_db 链路是教科书式三层（service/account_service.rs 四处同构的纯编排包装）。

**不满足点：**

- `refresh_master` 整个业务流程（两次 HTTP 外呼 + 拼响应）写在 handler
  （api/system_manage.rs:16-58），清洗层、编排层全部缺失（高）
- 编排层内联原子操作：`service/script_caller.rs:62-68` 直接 `Command` 派生 python 子进程、
  :114 直接 `ls` 列目录，无独立执行器原子模块（中）
- script 链路伪清洗：`req["api_name"].as_str().unwrap()`，缺字段 panic
  （api/script_caller.rs:16-19, 29-30）（中）
- `get_all` 无清洗，Query map 穿透三层直达 model 拼动态查询（中）
- 原子层直接返回 `serde_json::Value`，序列化渗入 model（model/cloud_account.rs:96-106）（低）

**独有特点：** api 与 service 存在同名模块 `script_caller`；云 API 调用 = 本地 python 脚本 +
子进程 argv 传 AK/SK（凭据明文上命令行，顺带的安全隐患）；脚本路径按 `api_name` 下划线
分段推导。

### 4. jc-commander —— 不满足

简单 CRUD 端点是标准三层透传，但核心业务流程全部越层：

- handler 直接 `create_channel` + `basic_publish` 做 MQ 投递
  （api/script_caller.rs:26,71-82,169,213-221）（高）
- `call_sync` 的完整编排（拼 payload→写日志→发 MQ→5ms 轮询 done_task_list→查结果→拼响应）
  全在 handler（api/script_caller.rs:20-161）（高）
- `refresh_master` handler 内两次 ureq 外呼，无 service/model（api/system_manage.rs:15-47）（高）
- `cron_job::new` handler 内手工组装两个业务实体、生成 uuid、填默认值
  （api/cron_job.rs:36-126）（中）
- `util/scheduler.rs` 越过 service 直持 db_pool/mq_pool，内含"发 MQ→next_round→置 flag"
  编排（:141-179, 224-256）（中）
- MQ 无统一封装，同一队列名 `{prefix}_async` 在两处各自拼装
  （api/script_caller.rs:176 vs util/scheduler.rs:227-234）（低）

**独有特点：** 三级时间轮调度器（秒/分/时槽位级联，main.rs:85 每秒 tick）；同步任务等待靠
全局 `SegQueue<Uuid>` + worker HTTP 回调 `/api/log/update` 入队通知，commander 无 MQ 消费者，
结果回收全走 HTTP 回调；cron 创建同一 uuid 双写 cron_job 与 job_log 两表（无事务）。

### 5. jc-worker —— 不满足（层次坍缩）

全 crate 仅 3 个有效源码文件，唯一流程为 MQ 消费循环：

```
main.rs:182-198 消费循环（MQ 原子操作全部内联在 main，无封装）
  → service/task.rs:9 execute（清洗+业务规则+进程执行全在一个函数）
  → service/json_rpc.rs:7 update_log（HTTP 外呼，命名为 service 实为原子+业务混合）
```

**不满足点：**

- 清洗层缺失：无强类型输入结构，`Value` + 链式 unwrap/expect，脏消息直接 panic 且发生在
  ack 之前，会拖垮消费循环（service/task.rs:10-36、main.rs:185-186）（高）
- 编排函数内含原子操作：直接 fork python 子进程（service/task.rs:47-51）（中）
- MQ 原子操作（connect/declare/consume/ack）全部内联在 main.rs:42-198（中）
- service 间互调（task → json_rpc），层次倒置为 service→service（低）

**独有特点：** 业务逻辑实际在 `script/` 下的 python 脚本里，Rust 侧只是"脚本 runner +
日志回报器"，因此"原子层 = DB CRUD"的预设在这里需替换为"原子层 = 进程执行/MQ/HTTP"；
通信是 MQ 下行 + HTTP 上行回调的混合模式。

### 6. file-agent —— 不满足（两层扁平结构）

实际是 `api handler → service` 两层结构，且两层都混入原子操作：

- upload handler 内嵌完整 multipart 解析 + `File::create` + 分块写入 + 限额判断
  （api/file_manage.rs:68-146，原子 IO 在 :94, :119）（高）
- `refresh_master` handler 内两次 HTTP 外呼编排（api/system_manage.rs:16, 46）（高）
- service 层直接执行 `std::fs` 原子操作（read_dir/read/remove_file/create_dir_all，
  service/dir_manage.rs:56,68、service/file_manage.rs:44,58,74），无独立原子层（高）
- 预期的原子层 `util/file_op.rs` 名不副实：无任何文件 IO，只有未完成的 token 鉴权
  数据结构，且全 crate 无人消费（util/file_op.rs:5-49、main.rs:59）（中）
- 未遵循 `from_map` 清洗惯例，`data.get("file").unwrap()` 裸取（中）

**独有特点：** 无 model 无数据库；service 文件用 `_manage` 后缀且与 api 层同名，是 handler
的机械"下半身"而非独立编排层；鉴权体系整体烂尾（中间件被注释、token 端点返回 418 彩蛋）。

### 7. yell —— 部分满足

发送链路有清晰的三层骨架：handler → `NotificationRouter`（渠道注册、并发 fan-out、
结果聚合，教科书式编排层）→ Channel trait 各渠道实现。渠道互不调用，trait object
设计干净，新渠道只需实现 trait + 注册一行。

**不满足点：**

- 9 个管理端点（template/record/channel/alias CRUD）跳过编排层，handler 直接
  `web::block` 调 model（api/notify.rs:278, 338, 407, 456, 498, 529, 597, 640, 697,
  755, 799）（高）
- handler 内含 DB 原子操作：`resolve_recipients` 直接查 alias（api/notify.rs:34-42）（高）
- 编排层直接查库：router 自己 `web::block` 查 ChannelConfig 和模板
  （service/notification_router.rs:233-260, 138-146）（中）
- 渠道 service 是"原子+编排"混合体，且 [查配置→建记录→外呼→更新记录] 小编排在
  5 渠道 × 2 方法（send/send_template）中近乎平行复制（infra/bark/service.rs:42-265 等）（中）
- model 层混入业务逻辑：模板 `render` 递归渲染 `{{var}}`（model/notification_template.rs:185-237）；
  ChannelConfig 做配置反序列化与业务错误（model/channel_config.rs:159-243）（中）
- 未遵循 `from_map` 清洗惯例，handler 逐字段手工摘取且含默认策略业务规则
  （api/notify.rs:164-223）（中）

**独有特点：** Channel trait + 注册表 + 每 recipient 一个 tokio::spawn 并发 fan-out；
渠道配置按 `(channel_type, name)` 支持同渠道多实例；模板按渠道分列 JSONB；统一发送审计
（先落 pending 记录再更新 sent/failed）；wecom 目录是未注册的空死代码。

## 共性偏差模式（跨 crate 系统性发现）

1. **`refresh_master` 集体越层**：cmdb、cloud-api、jc-commander、file-agent 四个 crate 的
   `api/system_manage.rs` 都在 handler 里直接做 HTTP 外呼 + 两步编排，全仓库同一份
   复制粘贴的违规。这是最一致、最适合优先收敛的模式。
2. **原子操作未按类型收敛**：DB（diesel）收敛最好（全部在 model/）；HTTP 外呼（ureq）
   散在 service/handler；MQ（lapin）散在 handler/util/main；进程派生（Command）散在
   service。缺的本质是"非 DB 原子操作没有归属层"。
3. **清洗层强度不均**：watchman/cmdb 的 POST 链路有 `from_map` 惯例，但所有 GET 链路的
   filter map 都穿透三层直达 model 拼动态查询；cloud-api/file-agent/yell 连 POST 都是
   裸 `unwrap` 取参，脏输入 panic 而非 400。
4. **`from_map` 住在 model 文件里**：清洗逻辑的物理位置在原子层文件，被 api 层调用，
   是全仓库统一的"折中惯例"，目标模型下应考虑独立出来。
5. **编排层两种极端**：要么空心化（cmdb 纯透传、jc-commander 名义存在），要么越界做原子
   操作（watchman HTTP 外呼、yell router 查库、file-agent service 做 fs）。
6. **middleware 是隐含的第四层**：watchman 的鉴权中间件内嵌多步 model 编排，
   与 service 逻辑重复，三层模型未覆盖横切层的职责约定。

## 顺手记录的非分层问题（疑似 bug）

- watchman signup 清洗时把 `username` 赋给了 `passwd` 字段（api/account_manage.rs:69-76）
- cmdb `from_map` 读取 `"plantform"` 拼写错误导致 platform 字段永远为 None
  （model/cloudserver/instance.rs:68）
- jc-commander `times` 字段误读 `map.get("frequency")`（model/cron_job.rs:53）；
  `call_async` 的 exec_type 硬编码为 `"sync"`（api/script_caller.rs:186）
- jc-worker `output` 被借用后二次 `unwrap`，stdout/stderr 皆空时会 panic
  （service/task.rs:53-94）

## 整改记录（2026-08-26）

本报告上文的各 crate「不满足点」清单为整改前快照，保留作历史对照。本轮整改已按下述主题完成。

### P0 共性收敛

- `share-lib` 新增 `infrastructure/` 原子操作层：`http_client`（ureq）、`mq_client`（lapin）、`process_runner`（std::process）、`master_registry`（向 watchman 注册/刷新），统一承接非 DB 原子操作。
- cmdb、cloud-api、jc-commander、file-agent 四个 crate 的 `refresh_master` 收敛到 `master_registry`，消除 handler 内复制粘贴的 HTTP 外呼 + 两步编排。
- watchman 的 ureq 外呼迁移至 `http_client`。
- 编排层全量 async 化，统一 tokio 异步形态（yell 本就合规，未改动）。

### P1 分层整改

- **jc-commander**：MQ 调用收敛至 `mq_client`；编排逻辑下沉 service；cron 组装从 handler 下沉。
- **jc-worker**：新增 TaskPayload 清洗层；MQ 消费循环独立为 `mq_consumer` 模块；`log_update` 归位 `util/`。
- **file-agent**：文件系统原语收敛为 `util/file_op.rs` 原子层；multipart 上传逻辑下沉 service；请求参数改为校验式清洗。
- **yell**：新增 `manage_service` / `notify_service` 编排层；`NotificationRouter` 纯化为注册与分发；`Channel` trait 收敛为单一 `dispatch` 职责；模板渲染逻辑移出 model；`wecom` 渠道删除。
- **cloud-api**：进程派生收敛至 `process_runner`；裸 `unwrap` 取参改为校验式清洗。
- **watchman**：删除 legacy `Authentication` 中间件及引用它的死代码。
- **cmdb**：改为校验式清洗；删除空文件。

### 清洗层统一

- 各 crate 新增 `api/filter.rs`，按表白名单校验 GET filter 的 key 与值类型，脏输入剔除而非 panic。
- yell send 链路改为结构化清洗，取代裸 `unwrap`。
- `from_map` 清洗函数留在 model 文件，确立为正式惯例（见 `doc/project_skeleton-CN.md`「清洗层的职责划分」）。

### 遗留与保留项

- 本机 Windows 环境缺 OpenSSL（`pq-sys` bundled 依赖链），5 个 diesel crate 的编译验证交由 `.github/workflows/check.yml` 执行，首次 push 即触发。
- clippy 基线警告未清零，暂不开启 `-D warnings`。
- middleware 与 `get_me` 的鉴权查询重复经评估不值得收敛，维持现状。
- AK/SK 通过 argv 传递凭据存在泄露风险，已记录，待后续协议变更解决。
