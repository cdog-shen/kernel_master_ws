<!-- filepath: /Users/cdog/Desktop/Projects/rs_ws/kernel_master_ws/watchman-backend/Readme_ZH-CN.md -->
# Watchman-backend

watchman 是整个 Kernel master 项目的 IAM 和调度服务.

它应该是 Kernel master 组件中第一个启动的服务.

## 代码规则

- 所有 ORM 模型对应的操作方法应放置在 ***model*** 目录下的相应模块中.

    通过 `from map` 进行输入映射, 未传入的值会被处理为 `None` .

    尽量使用 `Info/Model` 两个模型进行输入和输出, `User` 结构除外, 因为 `passwd` 字段不能对外输出.

    CURD方法只做基本实现, 新建方法中可适当放入字段校验和默认值, 这部分方法需要接受 `ref` 参数, 防止在 `Servce` 层中复用困难.

- 所有接口处理逻辑应放置在 ***service*** 目录下.

    接口主要逻辑在该层级, 直接接受参数而非 `ref` , 负责调用其他模块逻辑.

- 与 API 响应相关的逻辑应放置在 ***api*** 目录下的相应 API 模块中.

    所有查询接口方法应为 `GET`, 更新接口应为 `POST`, 删除接口应为 `DELETE`.

    该层级不进行逻辑处理, 只做基本数据处理, 确保 `Service` 层可以直接使用数据.

## APIs

所有 API 以 `/api` 范围开头.

### /hey

| 资源  |  支持的方法  | 功能                        | 备注         |
| :---: | :----------: | :-------------------------- | :----------- |
|   /   | `POST`/`GET` | 返回 `hi hello!` 原始字符串 | 服务测试 API |

### /reload

| 资源  | 支持的方法 | 功能             | 备注                         |
| :---: | :--------: | :--------------- | :--------------------------- |
|   /   |   `POST`   | 重新加载配置数据 | 热加载配置以刷新任何动态配置; 需要认证 |

### /auth

|   资源   | 支持的方法 | 功能                 | 备注                                                  |
| :------: | :--------: | :------------------- | :---------------------------------------------------- |
| /me/{id} |   `GET`    | 获取用户的所有信息   | 包含用户的所有信息                                    |
|  /login  |   `POST`   | 使用用户名和密码登录 |                                                       |
| /logout  |   `POST`   | 注销                 |                                                       |
| /verify  |   `POST`   | 子系统回源鉴权       | Bearer 携带用户 JWT; body 含子系统 name/uuid 与目标 path/method |


### /user

| 资源  | 支持的方法 | 功能             | 备注 |
| :---: | :--------: | :--------------- | :--- |
|   /   |   `GET`    | 获取所有用户信息 |      |
|   /   |   `POST`   | 注册             |      |
|   /   |  `PATCH`   | 更新用户信息     |      |

### /group

| 资源  | 支持的方法 | 功能           | 备注 |
| :---: | :--------: | :------------- | :--- |
|   /   |   `GET`    | 获取所有组信息 |      |
|   /   |   `POST`   | 创建一个组     |      |
|   /   |  `PATCH`   | 更新一个组     |      |
|   /   |  `DELETE`  | 删除一个组     |      |

### /service

| 资源  | 支持的方法 | 功能             | 备注                   |
| :---: | :--------: | :--------------- | :--------------------- |
|   /   |   `GET`    | 获取所有服务信息 |                        |
|   /   |   `POST`   | 创建一个服务     |                        |
|   /   |  `PATCH`   | 更新一个服务     |                        |
|   /   |  `DELETE`  | 删除一个服务     | 它还会禁用任何相关访问 |

### /access

| 资源  | 支持的方法 | 功能             | 备注 |
| :---: | :--------: | :--------------- | :--- |
|   /   |   `GET`    | 获取所有访问信息 |      |
|   /   |   `POST`   | 创建一个访问     |      |
|   /   |  `PATCH`   | 更新一个访问     |      |
|   /   |  `DELETE`  | 删除一个访问     |      |

### /subsystem

| 资源  | 支持的方法 | 功能               | 备注                                     |
| :---: | :--------: | :----------------- | :--------------------------------------- |
|   /   |   `GET`    | 获取所有子系统信息 |                                          |
|   /   |   `POST`   | 创建一个子系统     | 并且创建绑定服务                         |
|   /   |  `PATCH`   | 更新一个子系统     |                                          |
|   /   |  `DELETE`  | 删除一个子系统     | 它还会禁用任何相关访问并删除任何相关服务 |

### /subsystem_call

|       资源        | 支持的方法 | 功能                     | 备注                                                         |
| :---------------: | :--------: | :----------------------- | :----------------------------------------------------------- |
| /{subsystem_name} |   `POST`   | 使用 JSON 调用子系统服务 | 如果返回的不是 JSON, 它将被 JSON 化为 `{"data":"any data" }` |

## 数据库结构

- 用户表

    |  id   |     用户      |     密码      | 是否启用 |     名称      |        联系方式         |        加入日期         |        最后登录         |
    | :---: | :-----------: | :-----------: | :------: | :-----------: | :---------------------: | :---------------------: | :---------------------: |
    | uint  | varchar - str | varchar - str | tinyint  | varchar - str |          JSON           |        datetime         |        datetime         |
    |   0   |   testuser    |   00000000    |    1     |     test      | {email:"test@test.com"} | 2024-10-25 00:00:00.000 | 2024-11-11 09:15:26.978 |

- 令牌表

    |     用户      |         令牌          |                过期时间                 |
    | :-----------: | :-------------------: | :-------------------------------------: |
    | varchar - str |     varchar - str     |              datetime/int               |
    |     test      | ahsodhajkshdkanshdjka | 2024-10-25 00:00:00.000/UNIX_TIME_STAMP |

- 组表

    |  id   |     名称      | 是否启用 | 用户 ID 列表 |        更新日期         |
    | :---: | :-----------: | :------: | :----------: | :---------------------: |
    |  int  | varchar - str | tinyint  |     JSON     |        datetime         |
    |   0   |      dev      |    0     |  [1,2,3,4]   | 2024-10-25 00:00:00.000 |

- 服务表

    |  id   |   服务名称    |    服务点     | 是否启用 |        更新日期         |
    | :---: | :-----------: | :-----------: | :------: | :---------------------: |
    |  int  | varchar - str | varchar - str | tinyint  |        datetime         |
    |   0   |     CMDB      | /an/api/route |    0     | 2024-10-25 00:00:00.000 |

- 访问表

    |  id   | 服务 ID | 访问 ID | 组访问  | 是否启用 |        更新日期         |
    | :---: | :-----: | :-----: | :-----: | :------: | :---------------------: |
    |  int  |   int   |   int   | tinyint | tinyint  |        datetime         |
    |   0   |    0    |    0    | accINT  |    0     | 2024-10-25 00:00:00.000 |

- 子系统表

    |  id   |  uuid   | 服务名称 |              URL              |   是否启用    |        更新日期         | 相关服务 |
    | :---: | :-----: | :------: | :---------------------------: | :-----------: | :---------------------: | :------: |
    |  int  | varchar | varchar  |            varchar            | varchar - str |         tinyint         | datetime | int |
    |   0   | XXXXXXX | unnamed  | http://127.0.0.1:8000/api/hey |       0       | 2024-10-25 00:00:00.000 |    0     |

## 部署 和 依赖

- 密钥生成

    在启动服务之前, 生成一个随机密钥用于加密JWT. (也可以没有, 系统自动随机密钥)

    ```sh
    openssl rand -hex 32 > key/jwt_secret.key
    ```

- 依赖库

    - openssl
    - libpq

- 依赖服务

    - PostgreSQL

### 配置文件

配置文件名为 `watchman.toml`.

```toml
# 服务配置
[server_config]
# 日志配置
log_path = "log/watchman.log"
log_level = "DEBUG"    
clear_log = "True"
# 服务参数
listen_addr = "0.0.0.0"
listen_port = 8000
workers = 2
# CORS
allowed_origin_list = ["http://localhost:3000", "http://127.0.0.1:3000"]
# JWT secret
secret_key_path = "key/jwt_secret.key"
# 认证和鉴权白名单
authenticate_bypass = [
    "/api/hey",
    "/webhook",
    "/api/auth/login",
    "/api/auth/signup",
    "/api/subsystem_control/all_subsystem",
    "/api/subsystem_control/update_subsystem",
]
permit_bypass = [
    "/api/hey",
    "/webhook",
    "/api/auth/me",
    "/api/auth/login",
    "/api/auth/signup",
    # 回源鉴权端点自身跳过权限判定（JWT 仍须验）：真正判定的是 body 里的目标 path
    "/api/auth/verify",
    "/api/subsystem_control/all_subsystem",
    "/api/subsystem_control/update_subsystem",
]

# 数据库配置
[db_config]
db_str = "postgres://postgres:password@localhost:5432/watch_man"
```

# Q & A

- Q: 在高负载工况下watchman有时候对子系统RPC的请求会僵死

    直接添加更多的work线程就好, 不会拉高负载的
