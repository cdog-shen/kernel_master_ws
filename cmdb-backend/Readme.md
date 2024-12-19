# cmdb-backend

The CMDB service is responsible for managing all configuration information, which is utilized by other logic.

## Dependence

- std dependencies
    - once_cell = "1.20.2"
    - log = "0.4.22"
    - chrono = { version = "0.4.39", features = ["serde"] }
- basic web dependencies
    - actix-web = "4.9.0"
    - actix-rt = "2.10.0"
    - actix-service = "2.0.2"
    - actix-cors = "0.7.0"
    - futures = "0.3.31"
- DB dependencies
    - diesel_migrations = "2.2.0"
    - diesel = { version = "2.2.6", features = ["mysql", "r2d2", "chrono"] }
- serializtion dependencies
    - serde = "1.0.216"
    - serde_derive = "1.0.216"
    - serde_json = "1.0.133"
    - uuid = { version = "1.11.0", features = ["v4"] }
- RPC dependencies
    - ureq = { version = "2.12.1", features = ["charset", "json"] }
- share-lib utils
    - share-lib = { path = "../share-lib" }

## DB structure

## Deployment

### Config file

The name of the configuration file is `cmdb_server.cfg`.

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
