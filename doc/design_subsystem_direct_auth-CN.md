# 子系统直连 + 回源鉴权改造方案

> 状态：已实施（2026-08-28；迁移步骤 1-3 完成，步骤 4-5 待办）
> 决策记录：不做权限缓存（避免滞后与复杂度），每次请求由子系统向 watchman 回源鉴权

## 1. 背景与目标

现状链路：用户 → watchman（JWT 鉴权 + 权限判定）→ watchman 转发 HTTP 到子系统
（`POST /api/subsystem_call/{name}/{operate}`），子系统只校验 `Authorization: uuid
<subsys_uuid>` 确认调用方是中枢。

问题：

- watchman 是同步转发代理，其延迟/可用性叠加在每个业务请求上
- file-agent 的文件上传下载等大流量路径经代理转发，开销与稳定性都差
- 子系统对中枢的信任是裸 uuid header，无用户级语义

目标链路：用户持 JWT **直连子系统**；子系统取 JWT 向 watchman **回源鉴权**，
通过后本地执行。

原则：

- 不做权限/JWT 缓存，回源鉴权逻辑简单、无滞后
- 新旧链路并存过渡，调用方逐个迁移，最后下线转发链路
- 借此次改造收敛各 crate 已分化的 auth_middleware，统一进 share-lib

## 2. 链路对比

```
现状：
  用户 --JWT--> watchman [/api/subsystem_call] --uuid--> 子系统
                (JwtAuth + PermissionCheck)      (只验 uuid)

目标：
  用户 --JWT--> 子系统 --回源 verify(JWT, path, method)--> watchman [/api/auth/verify]
                (share-lib 鉴权中间件)                     (复用现有鉴权逻辑)
```

## 3. watchman 侧：新增 `POST /api/auth/verify`

### 接口定义

- 请求方身份：仅允许已注册子系统调用，调用方携带 `Authorization: uuid <subsys_uuid>`
  自证（与现有子系统间信任机制一致）；该接口不面向终端用户
- 请求体：

```json
{
  "token": "<用户 JWT>",
  "path": "/api/table/new",
  "method": "POST"
}
```

- 响应（MailMan 体系）：
  - 通过：`200 {code:200, key:"...", data:{"uid": 1, "username": "...", "permission": 2}}`
  - token 无效/过期/不在 token 表：`401`
  - 用户对该资源无权限：`403`

### 实现要点

- 复用 `JwtAuth` 中间件的现有逻辑（`UserToken::decode_token` + `TokenModel::token_ckeck`
  查 token 表）与 `PermissionCheck` 的权限判定（`GroupModel::get_groups_by_uid` →
  `ServiceModel::get_sids_by_route` → `AccessModel::get_max_permission`），
  抽取为 `service/auth_service.rs` 的 `verify(token, path, method)` 编排函数，
  中间件与 verify 接口共用同一实现，消除目前中间件与 service 的逻辑重复
- **权限粒度（MVP 简化）**：现有权限模型按"路由→service 记录"匹配。子系统路由
  数量多且不由 watchman 管理，MVP 采用**子系统级粒度**：每个子系统在 service 表
  对应一条记录，`service_point` 存子系统路由前缀（如 cmdb 的 `/api/table`），
  `get_sids_by_route` 前缀匹配即命中。权限语义不变：2=读写、1=只读（GET 放行）
- 路由注册：`/api/auth/verify` 加入 watchman 路由表；对子系统 uuid 的校验
  不走 bypass，走"子系统认证"（校验 uuid 是否存在于已启用子系统表）

## 4. 子系统侧：share-lib 统一鉴权中间件

各 crate 的 auth_middleware 已分化（uuid 校验细节各异），本次收敛为 share-lib
单一实现（feature `web` + `http`），替换各 crate 本地文件：

中间件逻辑（`share-lib/src/middleware/user_auth.rs`，新增）：

1. OPTIONS 与 `authenticate_bypass` 白名单放行（沿用现有语义）
2. 取 `Authorization` header：
   - `Bearer <jwt>`（新链路，用户直连）→ 调 watchman verify 回源鉴权，
     通过则把 `uid` 写入 request extensions 放行
   - `uuid <subsys_uuid>`（旧链路，watchman 转发）→ 按各 crate 现有逻辑
     校验本机 subsys_uuid 放行（过渡期保留，下线时删除该分支）
3. 失败按 MailMan 体系返回 401/403

配置：watchman 地址复用现有 `master_addr`/`master_port`（refresh_master 同款配置，
无需新增配置项）；verify 超时建议 3-5s，超时按 503 处理。

各 crate 接入点：

| crate | 现状 | 改动 |
|---|---|---|
| cmdb / cloud-api / jc-commander / yell | uuid 校验中间件已挂载 | 换 share-lib 中间件 |
| file-agent | 中间件被注释（无鉴权） | 借机启用 share-lib 中间件 |
| watchman | JwtAuth + PermissionCheck | 不动（它是鉴权源） |

## 5. 服务发现（用户如何拿到子系统 URL）

- watchman 的 subsys 注册表已有各子系统 `url` 字段（refresh_master 注册时上报）
- MVP：调用方登录后通过 watchman 现有子系统查询接口获取子系统 URL 清单，
  之后直连；前端侧实现不在本方案范围

## 6. 迁移步骤（分阶段，每阶段可独立部署）

1. ✅ **watchman**（已完成）：抽取 `auth_service::verify` + 新增 `/api/auth/verify` 接口；
   存量行为不变
2. ✅ **share-lib**（已完成）：新增统一鉴权中间件（Bearer 回源 + uuid 旧分支），
   见 `share-lib/src/middleware/user_auth.rs`
3. ✅ **子系统逐个接入**（已完成）：cmdb / cloud-api / jc-commander / yell / file-agent
   均已换用 share-lib 中间件并接入 `individual` feature，file-agent 借机启用鉴权；
   新旧链路并存
4. ⬜ **调用方切换直连**（待办）：前端/脚本改为直连子系统 URL
5. ⬜ **下线旧链路**（待办）：删除 watchman `subsystem_call` 转发与子系统 uuid 分支
   （本方案不含，待迁移完成后另行立项）

## 7. 风险与权衡（如实记录）

- **watchman 仍是可用性要害**：回源鉴权意味着 watchman 宕机时所有直连请求
  鉴权失败。相比转发模式的收益是：少一跳转发开销、大流量不过代理、职责清晰；
  但单点依赖未消除（已决策接受，后续如需可再加短 TTL 缓存）
- **verify QPS**：等于全系统业务 QPS。watchman 侧 verify 是两次 DB 查询
  （token 表 + 权限链），需关注 PG 连接池容量
- **权限粒度粗化**：MVP 为子系统级，路由级权限需子系统上报路由表，后续迭代
- **回源通道安全**：verify 接口依赖 subsys_uuid 保密；uuid 泄露 = 可伪造中枢。
  后续可考虑子系统与 watchman 间 mTLS 或密钥轮换，本期不做
- **token 撤销实时性**：回源模式天然支持（token 表删除即生效），优于缓存方案

## 8. 条件编译：`individual` 独立运行模式

每个子系统 crate 增加 feature `individual`（默认关闭）：

```sh
cargo build -p cmdb-backend                    # 默认：回源鉴权，依赖 watchman
cargo build -p cmdb-backend --features individual   # 独立模式：无需 watchman
```

语义：

- **individual 开启**：子系统编译为可脱离 watchman 独立运行的形态，认证方式回退为
  当前的 uuid 认证（`Authorization: uuid <subsys_uuid>` 与本机配置比对），
  回源鉴权代码**编译期裁掉**，运行时不需要 watchman 地址配置
- **individual 关闭（默认）**：回源鉴权（本方案主链路），过渡期同时保留 uuid 分支

实现要点：

- feature 传递链：子系统 crate 定义 `individual = ["share-lib/individual"]`，
  share-lib 的鉴权中间件内部按 `#[cfg(feature = "individual")]` 裁剪分支
  （回源分支 / uuid 分支二选一编译）
- `refresh_master` 端点与主关注册逻辑在 individual 模式下无意义，
  `config/app.rs` 中对应路由同样用 cfg 门控裁掉
- 配置：`individual` 模式下 `master_addr`/`master_port` 允许缺省（config 加载
  对这两个字段在该 feature 下放行）
- CI/验证矩阵：每个子系统 crate 都要覆盖 `--features individual` 与默认两种组合的
  `cargo check`（根 Makefile 或 build 脚本增加 `check --workspace --features individual`
  的等价命令；注意 workspace 级 feature 需用 `-p` 逐 crate 指定）
- docker-compose：后续可为需要的子系统增加 individual 构建 profile（本期不做）

## 9. 工作量预估

| 项 | 范围 | 量级 |
|---|---|---|
| watchman verify 接口 | auth_service 抽取 + 1 端点 + 路由 | 小 |
| share-lib 鉴权中间件 | 新模块（Transform/Service 样板 + 回源调用 + cfg 双分支） | 中 |
| 5 个子系统接入 | 换中间件 + individual feature 接线 + 配置 + 回归 | 中（重复性） |
| 文档/规范同步 | 骨架规范、AGENTS.md 链路描述 | 小 |

## 10. 实施偏差记录（实施时补充，与上文设计稿的差异如实记录）

- **verify 路由的精确挂法**：`/api/auth/verify` 注册为 `/api` scope **之前**的独立
  精确 resource（`web::resource("/api/auth/verify")`，见
  `watchman-backend/src/config/app.rs:15`），而非挂在 scope 内。原因：actix 路由按注册
  顺序命中，`/api` scope 前缀命中后内部失配不会回退到同级条目；且该端点自带子系统
  uuid 调用方认证，不能经过 scope 上挂的 `JwtAuth`/`PermissionCheck`。
- **auth_service 拆分粒度**：实际抽取为 `authenticate`（token 认证）/`authorize`
  （权限判定）/`verify`（编排入口）三段，中间件与 verify 接口共用同一实现，比设计稿的
  单函数更细。
- **http_client 增强**：为回源调用新增全局超时（连接 3s / 整体 5s，防对端挂起拖垮
  线程）与 `post_json_with_status`（返回 `(状态码, 响应体)`）；ureq 的"非 2xx 转 Error"
  已关闭（`http_status_as_error(false)`），状态码改由中间件分流：回源 401/403 原样映射，
  其余非 2xx、网络失败/超时一律 503。
- **回源调用的线程模型**：ureq 为同步阻塞实现，中间件内经 `actix_web::web::block`
  丢到 blocking 线程池执行，避免卡住 actix worker。
- **错误文案统一化**：中间件短路返回的 MailManErr 文案为统一英文（如
  "missing or invalid authorization header"、"subsystem uuid mismatch"、
  "token invalid or expired"、"permission denied by master"、
  "master verify unavailable: ..."），与各 crate 原分散文案不同；wire 格式保持
  MailManErr 序列化 JSON，与各 crate 原中间件一致。
- **feature 接线细节**：share-lib 侧 `individual = ["web"]`；子系统侧
  `individual = ["share-lib/individual"]`；非 individual 且未开 `http` feature 时
  `compile_error!` 直接拒绝编译。`individual` 模式下 `UserAuthConfig` 的
  `master_addr`/`master_port` 字段编译期裁掉、不参与中间件构造；各 crate config 加载
  对这两个配置项的读取未统一做 cfg 门控（存量仍按模板原样读取）。
- **file-agent 启用鉴权**：原先中间件整体注释（无鉴权裸奔），本次接入 share-lib
  中间件后启用，属设计内项的落实。
- **refresh_master 路由不一致（存量）**：cmdb / yell / file-agent 为
  `/api/manage/refresh_master`，cloud-api / jc-commander 为 `/api/refresh_master`；
  各 crate 的 `authenticate_bypass` 白名单已按实际路由配置，未做统一。
