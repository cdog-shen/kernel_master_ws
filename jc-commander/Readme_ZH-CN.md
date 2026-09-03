# 任务中心 - 控制器

控制器用于将任务消息发布到消息队列并记录所有任务状态。

## 依赖

- 标准依赖
    once_cell = "1.21.1"
    log = "0.4.26"
    chrono = { version = "0.4.40", features = ["serde"] }

- 基本网络依赖
    actix-web = "4.10.2"
    actix-rt = "2.10.0"
    actix-service = "2.0.3"
    actix-cors = "0.7.1"
    futures = "0.3.31"

- 数据库依赖
    diesel_migrations = "2.2.0"
    diesel = { version = "2.2.8", features = ["mysql", "r2d2", "chrono"] }

- 序列化依赖
    serde = "1.0.219"
    serde_derive = "1.0.219"
    serde_json = "1.0.140"
    uuid = { version = "1.16.0", features = ["v4"] }

- RPC 依赖
    ureq = { version = "2.12.1", features = ["charset", "json"] }

- 消息队列依赖
    lapin = "2.5.1"

- 共享库工具
    share-lib = { path = "../share-lib" }

## 数据库结构

## 部署

- 依赖服务

    - MySQL >= 8.0

    - rabbitMQ >= 4.0

### 配置文件

配置文件的名称是 `job_center_commander.cfg`。

```toml
[server_config]
listen_addr = "127.0.0.1"
listen_port = 9003
log_path = "logs/commander.log"
log_level = "INFO"
clear_log = "True"
master_addr = "127.0.0.1"
master_port = 8000
register_name = "job-center"
authenticate_bypass = ["/api/hey", "/api/manage/refresh_master"]

[db_config]
db_str = "mysql://root:778631@127.0.0.1:3306/job_center"

[mq_config]
mq_str = "amqp://commander:commander@127.0.0.1/job_center"
queue_prefix = "job_queue"
```