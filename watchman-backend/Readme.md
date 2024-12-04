# Watchman-backend

watchman is the IAM and dispatch service for whole Kernel master project.

it should be the FIRST launched service in Kernel master compoment.

## Code rules

- All ORM model corresponding operation methods should be placed in the corresponding mod under the ***models*** directory.

- All interface processing logic should be placed in the ***services*** directory.

- The logic related to API responses should be placed under the corresponding API mod in the ***apis*** directory.

## Dependence

- basic web dependencies
    - actix-web = "4.9.0"
    - actix-rt = "2.10.0"
    - actix-service = "2.0.2"
    - actix-cors = "0.7.0"
    - futures = "0.3.31"
- DB dependencies
    - diesel_migrations = "2.2.0"
    - diesel = { version = "2.2.4", features = ["mysql", "r2d2", "chrono"] }
- serializtion dependencies
    - serde = "1.0.215"
    - serde_derive = "1.0.215"
    - serde_json = "1.0.132"
    - jsonwebtoken = "9.3.0"
    - bcrypt = "0.15.1"
    - base64 = "0.22.1"
- share-lib utils
    - share-lib = { path = "../share-lib" }

## Notes

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
