# Job center - Commander

The commander is used to publish task messages to the message queue and record all task status.

## Dependence

- std dependencies
    once_cell = "1.21.1"
    log = "0.4.26"
    chrono = { version = "0.4.40", features = ["serde"] }
    crossbeam = '0.8.4'

- basic web dependencies
    actix-web = "4.10.2"
    actix-rt = "2.10.0"
    actix-service = "2.0.3"
    actix-cors = "0.7.1"
    futures = "0.3.31"

- DB dependencies
    diesel_migrations = "2.2.0"
    diesel = { version = "2.2.8", features = ["mysql", "r2d2", "chrono"] }

- serializtion dependencies
    serde = "1.0.219"
    serde_derive = "1.0.219"
    serde_json = "1.0.140"
    uuid = { version = "1.16.0", features = ["v4"] }

- RPC dependencies
    ureq = { version = "2.12.1", features = ["charset", "json"] }

- MQ dependencies
    lapin = "2.5.1"

- share-lib utils
    share-lib = { path = "../share-lib" }

## DB structure

## Deployment

- dependence service

    - MySQL >= 8.0

    - rabbitMQ >= 4.0

### Config file

The name of the configuration file is `job_center_commander.cfg`.

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
authenticate_bypass = ["/api/refresh_master"]

[db_config]
db_str = "mysql://root:778631@127.0.0.1:3306/job_center"

[mq_config]
mq_str = "amqp://commander:commander@127.0.0.1/job_center"
queue_prefix = "job_queue"
```
