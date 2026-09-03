# cloud-api

Cloud API 服务负责管理云资源并为其他服务提供 API。

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
    - Python >= 3.9 (有相关需求)

### 配置文件

配置文件的名称是 `cloud_api_server.cfg`。

```toml
[server_config]
listen_addr = "127.0.0.1"
listen_port = 9002
log_path = "logs/cloudapi.log"
log_level = "INFO"
clear_log = "True"
master_addr = "127.0.0.1"
master_port = 8000
register_name = "cloud-api"
authenticate_bypass = ["/api/hey", "/api/manage/refresh_master"]
python_path = "/opt/miniconda3/envs/cloud_api/bin/python"
script_dir = "scripts"

[db_config]
db_str = "mysql://root:778631@127.0.0.1:3306/cloud_api"
```