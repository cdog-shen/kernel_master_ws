# 任务中心 - 执行器

该执行器从消息队列中读取任务信息并执行。

## 依赖

- 标准依赖
    once_cell = "1.21.1"
    log = "0.4.26"
    chrono = { version = "0.4.40", features = ["serde"] }
    futures = "0.3.31"
    tokio = { version = "1.44.1", features = ["full"] }

- 序列化依赖
    serde = "1.0.219"
    serde_derive = "1.0.219"
    serde_json = "1.0.140"
    uuid = { version = "1.16.0", features = ["v4"] }

- RPC 依赖
    ureq = { version = "3.0.9" }

- MQ 依赖
    lapin = "2.5.1"

- 共享库工具
    share-lib = { path = "../share-lib" }

## 数据库结构

执行器不直接连接到数据库。

## 部署

- 依赖服务

    - rabbitMQ >= 4.0
    - Python >= 3.9 (带有需求)

### 配置文件

配置文件的名称是 `job_center_worker.cfg`。

```toml
[server_config]
log_path = "log/commander.log"
commander_addr = "127.0.0.1:9003"
log_level = "INFO"
clear_log = "True"
python_path = "python"
script_dir = "script"

[mq_config]
mq_str = "amqp://commander:commander@127.0.0.1/job_center"
queue_prefix = "job_queue"
```