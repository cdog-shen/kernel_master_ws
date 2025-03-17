# Job center - Worker

The worker reads task information from the message queue and executes it.

## Dependence

- std dependencies
    once_cell = "1.21.1"
    log = "0.4.26"
    chrono = { version = "0.4.40", features = ["serde"] }
    futures = "0.3.31"
    tokio = { version = "1.44.1", features = ["full"] }

- serializtion dependencies
    serde = "1.0.219"
    serde_derive = "1.0.219"
    serde_json = "1.0.140"
    uuid = { version = "1.16.0", features = ["v4"] }

- RPC dependencies
    ureq = { version = "3.0.9" }

- MQ dependencies
    lapin = "2.5.1"

- share-lib utils
    share-lib = { path = "../share-lib" }


## DB structure

The worker is not directly connected to the database. 

## Deployment

### Config file

The name of the configuration file is `job_center_worker.cfg`.

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