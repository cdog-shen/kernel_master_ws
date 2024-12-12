# Watchman-backend

watchman is the IAM and dispatch service for whole Kernel master project.

it should be the FIRST launched service in Kernel master compoment.

## Code rules

- All ORM model corresponding operation methods should be placed in the corresponding mod under the ***models*** directory.

    Each data table corresponds to a model file, which contains three structures inside (Model/InputStream/OutputStream).

    Only the Model structure is implemented, with input through the InputStream structure and output results using the OutputStream structure. For convenience in outputting, a map_model_to_output_stream function should be defined for each Model.

    The Model structure has two impl blocks, one implementing all query methods and the other implementing all modification methods.

- All interface processing logic should be placed in the ***services*** directory.

    When inputting data structures, you should use `Models::table::InputStream`, or use other data types as needed.

- The logic related to API responses should be placed under the corresponding API mod in the ***apis*** directory.

    All query interface methods should be `GET`, update interfaces should be `POST`, and delete interfaces should be `DELETE`.

## Dependence

- basic support dependencies
    - once_cell = "1.20.2"
    - toml = "0.8.19"
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
    - jsonwebtoken = "9.3.0"
    - bcrypt = "0.16.0"
    - base64 = "0.22.1"
    - uuid = { version = "1.11.0", features = ["v4"] }
- share-lib utils
    - share-lib = { path = "../share-lib" }

## Deployment

## Config file

The name of the configuration file is `watchman_server.cfg`.

```toml
[server_config]
listen_addr = "127.0.0.1"
listen_port = 8080
log_path = "logs/watchman.log" # log file location
log_level = "INFO" # log level
secret_key_path = "keys/jwt_secret.key" # The key used for generating JWT.
clear_log = "True" # logs be cleared upon startup
authenticate_bypass = ["/api/auth/signup", "/api/auth/login", "/webhook"];
permit_bypass = ["/api/auth/signup", "/api/auth/login", "/webhook"]

[db_config] # DB string config
db_str = "mysql://root:778631@127.0.0.1:3306/watch_man" # local

[subsys_config]
key1 = "v1"
```

### Build diesel on Windows

1. download a mysql-community-server ZIP pac

    I choose the latest version [(9.1.0)](https://cdn.mysql.com/archives/mysql-9.0/mysql-9.0.1-winx64.zip)

    Or the C++ connector, I think it is OKEY but I'm too tired to try it `:(`

2. create a copy of **mysql-9.0.1-winx64\lib\mysqlclient.lib** and name it **mysql-9.0.1-winx64\lib\libmysqlclient.lib**

    Just into lib dir do this `cp mysqlclient.lib libmysqlclient.lib`

3. add 2 Environment variables

    `MYSQLCLIENT_LIB_DIR` = `C:\Program Files\MySQL\mysql-9.0.1-winx64\lib` (Change it to ur mysql path)

    `MYSQLCLIENT_VERSION` = `8.0.30` (I don't Know why they only support specifying a few versions !!! 8.0.30 it currently latest)

4. done. u can install diesel_cli with mysql feature

    `cargo install diesel_cli --no-default-features --features "mysql"`
