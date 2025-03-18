# cmdb-backend

CMDB 服务负责管理所有配置信息，供其他逻辑使用。

## 依赖

- 标准依赖
    - once_cell = "1.20.2"
    - log = "0.4.22"
    - chrono = { version = "0.4.39", features = ["serde"] }

- 基本 web 依赖
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
    - uuid = { version = "1.11.0", features = ["v4"] }

- RPC 依赖
    - ureq = { version = "2.12.1", features = ["charset", "json"] }

- 共享库工具
    - share-lib = { path = "../share-lib" }

## 数据库结构

## 部署

- 依赖服务

    - MySQL >= 8.0

### 配置文件

配置文件的名称是 `cmdb_server.cfg`。

```toml
[server_config]
listen_addr = "127.0.0.1"
listen_port = 9001
log_path = "logs/cmdb.log"
log_level = "INFO"
clear_log = "True"

[db_config]
db_str = "mysql://root:778631@127.0.0.1:3306/cmdb"
```