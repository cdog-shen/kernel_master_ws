# AGENTS.md

本文件是 kernel_master_ws 仓库的总纲，面向 AI 编码助手与新加入的开发者。
修改代码前请先读完本文件；涉及具体主题时，再查阅 `doc/` 下的专项规范（见文末索引）。

## 项目概述

kernel master 是一个 Rust 微服务式项目（毕业设计）。`watchman-backend` 是 IAM 与调度中枢。
调用模型双链路并存（过渡期，设计见 `doc/design_subsystem_direct_auth-CN.md`）：
用户持 JWT **直连子系统**，子系统向 watchman `POST /api/auth/verify` **回源鉴权**后本地执行；
旧的 `POST watchman/api/subsystem_call/{subsystem_name}/{operate}` 中枢转发链路在调用方
迁移完成前保留，最终下线。公共能力收敛在 `share-lib`（配置读取、日志、MailMan 消息结构、
web 错误响应映射、统一鉴权中间件）。

## 仓库布局（Cargo workspace）

根目录 `Cargo.toml` 定义 workspace（`resolver = "3"`），所有依赖版本统一在
`[workspace.dependencies]` 声明，成员 crate 一律用 `xxx.workspace = true` 继承。

| crate | 职责 | 端口 | 特点 |
|---|---|---|---|
| `share-lib` | 公共库：cfg_reader / data_structure / logger / err_mapping | — | 库 crate，无二进制 |
| `watchman-backend` | IAM、鉴权、子系统调度 | 8000 | 骨架最完整的参考实现 |
| `cmdb-backend` | 配置管理数据库（CMDB） | 9001 | model 按子系统分目录；有 `full-schema` feature |
| `cloud-api` | 云资产权限管理与调用 | 9002 | |
| `jc-commander` | Job-Center 调度端 | 9003 | RabbitMQ(lapin) + tokio |
| `jc-worker` | Job-Center 执行端 | — | 纯 MQ 消费者，无 actix/diesel |
| `file-agent` | 文件系统操作 | 9004 | multipart 上传，无数据库 |
| `yell` | 通知服务（bark/gotify/mail/teams/webhook/wecom） | 9005 | `services/` 复数目录，按渠道组织 |

## 构建与检查命令

在仓库根目录执行（workspace 级，优先于逐 crate 操作）：

```sh
cargo fmt --all                # 格式化（CI/提交前用 -- --check）
cargo check --workspace        # 快速检查
cargo clippy --workspace --all-targets   # lint（目前仅记录警告，未开 -D warnings）
cargo build --release --workspace        # 全量构建，产物在 target/release/
```

`build/` 下的脚本是对上述命令的封装，产出二进制统一归拢到 `./dist/`：

```sh
build/build_all_ws.sh        # release 构建 + 拷贝二进制到 ./dist/
build/check_all_ws.sh        # cargo check --workspace --all-targets
build/clippy_all_ws.sh       # cargo clippy --workspace --all-targets
build/fmt_all_ws.sh          # cargo fmt --all（参数透传，如 -- --check）
build/clean_all_ws.sh        # cargo clean
build/start.sh               # 本地后台拉起全部服务
```

Docker 方式（需要 Docker 环境）：

```sh
docker-compose --profile build up      # 用 builder 容器编译
docker-compose --profile run up -d     # 启动全部服务（先备好 MQ 与 DB，改好配置）
```

注意：`pq-sys` 使用 `bundled_without_openssl` feature（vendored libpq，不含 OpenSSL），
任何环境都无需安装 OpenSSL 开发包即可编译；代价是 PG 连接不支持 TLS（`sslmode=require`
不可用），如需加密连接请改回 `bundled` 并配置 OpenSSL/vcpkg。

各子系统 crate（cmdb-backend / cloud-api / jc-commander / yell / file-agent）另有
`individual` feature（`xxx = ["share-lib/individual"]`），开启后编译为脱离 watchman 的
独立运行模式：鉴权中间件编译期裁剪为仅 uuid 比对分支，refresh_master 路由一并裁掉。
workspace 级无该 feature，验证时需逐 crate 指定：

```sh
cargo check -p cmdb-backend --features individual
```

## 开发铁律

1. **错误必须走 MailMan 体系**：`MailManOk` / `MailManErr`（share-lib）→
   `MailManErrResponser`（share-lib `web` feature）转 HTTP 响应。细节见
   `doc/error_handling-CN.md`。
2. **公共代码优先进 share-lib**，禁止跨 crate 复制粘贴。历史教训：`err_mapping.rs`
   曾在 6 个 crate 逐字节重复，已收敛；鉴权中间件已收敛——子系统鉴权的唯一来源是
   `share-lib/src/middleware/user_auth.rs`（各子系统本地 `auth_middleware.rs` 已删除；
   watchman-backend 是鉴权源，保留本地 `JwtAuth`/`PermissionCheck`）；
   `config/server.rs` 各 crate 已分化（依赖自身 model），暂不抽取，改动时逐 crate 同步评估。
3. **新增 crate 必须完成全部登记**：根 `Cargo.toml` members、依赖走 workspace 继承、
   `build/*.sh` 的 DIRS/BINS、`docker-compose.yaml`（历史教训：yell 曾长期漏登记）。
4. **配置走 `GLOBAL_CONFIG` 模式**：`config/server.rs` + once_cell 全局单例 +
   `*.template.toml` 模板；`/api/reload` 热重载目前仅 watchman-backend 注册，其余 crate 按需接入。
5. **依赖版本只在根 `Cargo.toml` 改**，成员 crate 不得写版本号。
6. **提交前跑 `build/fmt_all_ws.sh -- --check` 和 `build/clippy_all_ws.sh`**。
7. 数据库迁移用 diesel CLI 手工执行（`migrations/` 目录），每个迁移必须有
   `up.sql` 和 `down.sql`。
8. 本期规约：暂不补单测、暂不新增业务代码；格式化已完成一次性格式化（`cargo fmt --all`）。

## 文档索引（doc/）

每篇均有中文版（`-CN.md`）与英文版（`-EN.md`）：

- `doc/coding_rules-CN.md` — 编码规范
- `doc/project_skeleton-CN.md` — 项目骨架规范
- `doc/error_handling-CN.md` — 错误处理规范
- `doc/logging-CN.md` — 日志规范
- `doc/comments-CN.md` — 注释规范
- `doc/naming-CN.md` — 命名规范
