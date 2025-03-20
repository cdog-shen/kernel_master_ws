<!-- filepath: /Users/cdog/Desktop/Projects/rs_ws/kernel_master_ws/watchman-backend/Readme_ZH-CN.md -->
# Watchman-backend

watchman 是整个 Kernel master 项目的 IAM 和调度服务。

它应该是 Kernel master 组件中第一个启动的服务。

## 代码规则

- 所有 ORM 模型对应的操作方法应放置在 ***models*** 目录下的相应模块中。

    每个数据表对应一个模型文件，其中包含三个结构（Model/Info）。

    只实现 Model 结构，通过 Info 结构输出， 通过JSON映射到Info结构进行输入。

    Model 结构有两个 impl 块，一个实现所有查询方法，另一个实现所有修改方法。

- 所有接口处理逻辑应放置在 ***services*** 目录下。

    输入数据结构时，应使用 `Models::table::InputStream`，或根据需要使用其他数据类型（推荐 `serde_json::Map<String, serde_json::Value>`）。

- 与 API 响应相关的逻辑应放置在 ***apis*** 目录下的相应 API 模块中。

    所有查询接口方法应为 `GET`，更新接口应为 `POST`，删除接口应为 `DELETE`。

    如果 POST 数据包含任何 JSON 对象，请在此文件中将其反序列化为字符串。

## APIs

所有 API 以 `/api` 范围开头。

### /hey

| 资源 | 支持的方法 | 功能                      | 备注              |
| :--: | :--------: | :------------------------ | :--------------- |
|  /   |  `POST`/`GET`  | 返回 `hi hello!` 原始字符串 | 服务测试 API     |

### /reload

| 资源 | 支持的方法 | 功能           | 备注                                         |
| :--: | :--------: | :------------- | :------------------------------------------ |
|  /   |   `POST`   | 重新加载配置数据 | 热加载配置以刷新任何动态配置                 |

### /auth

|   资源   | 支持的方法 | 功能                           | 备注                 |
| :------: | :--------: | :----------------------------- | :------------------ |
| /all_user |   `GET`    | 获取所有用户信息               |                      |
| /me/{id}  |   `GET`    | 获取用户的所有信息             | 包含用户的所有信息   |
| /signup   |   `POST`   | 注册                           |                      |
| /login    |   `POST`   | 使用用户名和密码登录           |                      |
| /logout   |   `POST`   | 注销                           |                      |
| /user_update | `POST`  | 更新用户信息                   |                      |

### /group_control

|   资源    | 支持的方法 | 功能                 | 备注 |
| :-------: | :--------: | :------------------- | :-- |
| /all_group |   `GET`    | 获取所有组信息       |      |
| /new_group |   `POST`   | 创建一个组           |      |
| /update_group | `POST`  | 更新一个组           |      |
| /delete_group | `DELETE`| 删除一个组           |      |

### /service_control

|    资源     | 支持的方法 | 功能                   | 备注                            |
| :---------: | :--------: | :--------------------- | :----------------------------- |
| /all_service |   `GET`    | 获取所有服务信息       |                                |
| /new_service |   `POST`   | 创建一个服务           |                                |
| /update_service | `POST`  | 更新一个服务           |                                |
| /delete_service | `DELETE`| 删除一个服务           | 它还会禁用任何相关访问          |

### /access_control

|    资源    | 支持的方法 | 功能                  | 备注 |
| :--------: | :--------: | :-------------------- | :-- |
| /all_access |   `GET`    | 获取所有访问信息       |      |
| /new_access |   `POST`   | 创建一个访问           |      |
| /update_access | `POST`  | 更新一个访问           |      |
| /delete_access | `DELETE`| 删除一个访问           |      |

### /subsystem_control

|     资源      | 支持的方法 | 功能                     | 备注                                                            |
| :-----------: | :--------: | :----------------------- | :------------------------------------------------------------- |
| /all_subsystem |   `GET`    | 获取所有子系统信息       |                                                                |
| /new_subsystem |   `POST`   | 创建一个子系统           | 并且创建绑定服务                                               |
| /update_subsystem | `POST`  | 更新一个子系统           |                                                                |
| /delete_subsystem | `DELETE`| 删除一个子系统           | 它还会禁用任何相关访问并删除任何相关服务                        |

### /subsystem_call

|     资源      | 支持的方法 | 功能                          | 备注                                                            |
| :-----------: | :--------: | :---------------------------- | :------------------------------------------------------------- |
| /{subsystem_name} | `POST`  | 使用 JSON 调用子系统服务       | 如果返回的不是 JSON，它将被 JSON 化为 `{"data":"any data" }`    |

## 依赖

- 基本支持依赖
    - once_cell = "1.20.2"
    - log = "0.4.22"
    - chrono = { version = "0.4.39", features = ["serde"] }
- 基本 Web 依赖
    - actix-web = "4.9.0"
    - actix-rt = "2.10.0"
    - actix-service = "2.0.2"
    - actix-cors = "0.7.0"
    - futures = "0.3.31"
- 数据库依赖
    - diesel_migrations = "2.2.0"
    - diesel = { version = "2.2.6", features = ["mysql", "r2d2", "chrono"] }
- 序列化依赖
    - serde = "1.0.216"
    - serde_derive = "1.0.216"
    - serde_json = "1.0.133"
    - jsonwebtoken = "9.3.0"
    - bcrypt = "0.16.0"
    - base64 = "0.22.1"
    - uuid = { version = "1.11.0", features = ["v4"] }
- 共享库工具
    - share-lib = { path = "../share-lib" }

## 数据库结构

- 用户表

    |  id   |     用户      |    密码     | 是否启用 |     名称      |         联系方式         |        加入日期         |         最后登录         |
    | :---: | :-----------: | :---------: | :------: | :-----------: | :---------------------: | :---------------------: | :---------------------: |
    | uint  | varchar - str | varchar - str | tinyint | varchar - str |          JSON           |          datetime       |          datetime       |
    |   0   |   testuser    |   00000000  |     1    |     test      | {email:"test@test.com"} | 2024-10-25 00:00:00.000 | 2024-11-11 09:15:26.978 |

- 令牌表

    |     用户      |         令牌         |                  过期时间                  |
    | :-----------: | :-------------------: | :----------------------------------------: |
    | varchar - str |     varchar - str     |                datetime/int                |
    |     test      | ahsodhajkshdkanshdjka | 2024-10-25 00:00:00.000/UNIX_TIME_STAMP    |

- 组表

    |  id   |     名称      | 是否启用 | 用户 ID 列表 |        更新日期         |
    | :---: | :-----------: | :------: | :----------: | :---------------------: |
    |  int  | varchar - str | tinyint  |     JSON     |          datetime       |
    |   0   |      dev      |     0    |  [1,2,3,4]   | 2024-10-25 00:00:00.000 |

- 服务表

    |  id   | 服务名称      | 服务点      | 是否启用 |        更新日期         |
    | :---: | :-----------: | :---------: | :------: | :---------------------: |
    |  int  | varchar - str | varchar - str | tinyint |          datetime       |
    |   0   |     CMDB      | /an/api/route |     0   | 2024-10-25 00:00:00.000 |

- 访问表

    |  id   | 服务 ID | 访问 ID | 组访问 | 是否启用 |        更新日期         |
    | :---: | :-----: | :-----: | :----: | :------: | :---------------------: |
    |  int  |   int   |   int   | tinyint | tinyint |          datetime       |
    |   0   |    0    |    0    | accINT |     0    | 2024-10-25 00:00:00.000 |

- 子系统表

    |  id   |  uuid   | 服务名称 |              URL              |   是否启用   |        更新日期         | 相关服务 |
    | :---: | :-----: | :------: | :---------------------------: | :----------: | :---------------------: | :------: |
    |  int  | varchar | varchar  |            varchar            | varchar - str |          tinyint        | datetime | int |
    |   0   | XXXXXXX | unnamed  | http://127.0.0.1:8000/api/hey |       0       | 2024-10-25 00:00:00.000 |    0     |

## 部署

- 密钥生成

    在启动服务之前，生成一个随机密钥用于加密JWT。

    ```sh
    openssl rand -hex 32 > key/jwt_secret.key
    ```

- 依赖服务

    - MySQL >= 8.0

### 配置文件

配置文件名为 `watchman_server.cfg`。

```toml
[server_config]
log_path = "logs/watchman.log"
log_level = "DEBUG"
listen_addr = "127.0.0.1"
listen_port = 8000
allowed_origin_list = ["http://localhost:3000", "http://127.0.0.1:3000"]
# pub_key_path = "keys/jwt_pub.der"
# pri_key_path = "keys/jwt_pri.der"
secret_key_path = "keys/jwt_secret.key"
clear_log = "True"
authenticate_bypass = ["/api/auth/signup","/api/auth/login","/webhook","/api/hey","/api/reload","/api/subsystem_control/all_subsystem","/api/subsystem_control/update_subsystem"]
permit_bypass = ["/api/auth/signup","/api/auth/login","/webhook","/api/hey","/api/reload","/api/subsystem_control/all_subsystem","/api/subsystem_control/update_subsystem"]

[db_config]
db_str = "mysql://root:778631@127.0.0.1:3306/watch_man" # local
```

### 在 Windows 上构建 diesel

1. 下载 mysql-community-server ZIP 包

    我选择了最新版本 [(9.1.0)](https://cdn.mysql.com/archives/mysql-9.0/mysql-9.0.1-winx64.zip)

    或者 `C++ connector`，我认为它是可以的，但我懒得试了 `:(`

2. 创建 **mysql-9.0.1-winx64\lib\mysqlclient.lib** 的副本，并将其命名为 **mysql-9.0.1-winx64\lib\libmysqlclient.lib**

    只需进入 lib 目录执行此操作 `cp mysqlclient.lib libmysqlclient.lib`

3. 添加 2 个环境变量

    `MYSQLCLIENT_LIB_DIR` = `C:\Program Files\MySQL\mysql-9.0.1-winx64\lib` （更改为你的 mysql 路径）

    `MYSQLCLIENT_VERSION` = `8.0.30` （我不知道为什么他们只支持指定几个版本！！！8.0.30 是目前最新的）

4. 完成。你可以安装带有 mysql 功能的 diesel_cli

    `cargo install diesel_cli --no-default-features --features "mysql"`