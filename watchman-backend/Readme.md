# Watchman-backend

watchman is the IAM and dispatch service for whole Kernel master project.

it should be the FIRST launched service in Kernel master compoment.

## Code rules

- All ORM model corresponding operation methods should be placed in the corresponding mod under the ***models*** directory.

    Each data table corresponds to a model file, which contains three structures inside (Model/InputStream/OutputStream).

    Only the Model structure is implemented, with input through the InputStream structure and output results using the OutputStream structure. For convenience in outputting, a map_model_to_output_stream function should be defined for each Model.

    The Model structure has two impl blocks, one implementing all query methods and the other implementing all modification methods.

- All interface processing logic should be placed in the ***services*** directory.

    When inputting data structures, you should use `Models::table::InputStream`, or use other data types(i recommand `serde_json::Map<String, serde_json::Value>`) as needed.

- The logic related to API responses should be placed under the corresponding API mod in the ***apis*** directory.

    All query interface methods should be `GET`, update interfaces should be `POST`, and delete interfaces should be `DELETE`.

    And if POST data include any JSON object, deserialize it to String in this file.

## APIs

all api started with `/api` scope.

### /hey

| resource | method support | function                      | comment              |
| :------: | :------------: | :---------------------------- | :------------------- |
|    /     |  `POST`/`GET`  | return `hi hello!` raw string | test API for service |


### /reload

| resource | method support | function           | comment                                         |
| :------: | :------------: | :----------------- | :---------------------------------------------- |
|    /     |     `POST`     | reload config data | Hot reload config to refresh any dynamic config |

### /auth

|   resource   | method support | function                           | comment                 |
| :----------: | :------------: | :--------------------------------- | :---------------------- |
|  /all_user   |     `GET`      | get all user info with get query   |                         |
|   /me/{id}   |     `GET`      | get user's all info with user's id | contain user's all info |
|   /signup    |     `POST`     | just a signup                      |                         |
|    /login    |     `POST`     | login with username and password   |                         |
|   /logout    |     `POST`     | logout                             |                         |
| /user_update |     `POST`     | update user's info                 |                         |

### /group_control

|   resource    | method support | function                 | comment |
| :-----------: | :------------: | :----------------------- | :------ |
|  /all_group   |     `GET`      | get all group with query |         |
|  /new_group   |     `POST`     | create a group           |         |
| /update_group |     `POST`     | update a group           |         |
| /delete_group |    `DELETE`    | delete a group           |         |

### /service_control

|    resource     | method support | function                   | comment                            |
| :-------------: | :------------: | :------------------------- | :--------------------------------- |
|  /all_service   |     `GET`      | get all service with query |                                    |
|  /new_service   |     `POST`     | create a service           |                                    |
| /update_service |     `POST`     | update a service           |                                    |
| /delete_service |    `DELETE`    | delete a service           | it also disable any related access |

### /access_control

|    resource    | method support | function                  | comment |
| :------------: | :------------: | :------------------------ | :------ |
|  /all_access   |     `GET`      | get all access with query |         |
|  /new_access   |     `POST`     | create a access           |         |
| /update_access |     `POST`     | update a access           |         |
| /delete_access |    `DELETE`    | delete a access           |         |

### /subsystem_control

|     resource      | method support | function                     | comment                                                            |
| :---------------: | :------------: | :--------------------------- | :----------------------------------------------------------------- |
|  /all_subsystem   |     `GET`      | get all subsystem with query |                                                                    |
|  /new_subsystem   |     `POST`     | create a subsystem           | and also create bind services                                      |
| /update_subsystem |     `POST`     | update a subsystem           |                                                                    |
| /delete_subsystem |    `DELETE`    | delete a subsystem           | it also disable any related access and delete any related services |

### /subsystem_call

|     resource      | method support | function                          | comment                                                            |
| :---------------: | :------------: | :-------------------------------- | :----------------------------------------------------------------- |
| /{subsystem_name} |     `POST`     | call subsystem services with JSON | if return not a JSON, it will be jsonify as `{"data":"any data" }` |

## Dependence

- basic support dependencies
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
    - jsonwebtoken = "9.3.0"
    - bcrypt = "0.16.0"
    - base64 = "0.22.1"
    - uuid = { version = "1.11.0", features = ["v4"] }
- share-lib utils
    - share-lib = { path = "../share-lib" }

## DB structure

- User table

    |  id   |     user      |    passwd     | is_enable |     name      |         contact         |        date_joined         |         last_login         |
    | :---: | :-----------: | :-----------: | :-------: | :-----------: | :---------------------: | :------------------------: | :------------------------: |
    | uint  | varchar - str | varchar - str |  tinyint  | varchar - str |          JSON           |          datetime          |          datetime          |
    |   0   |   testuser    |   00000000    |     1     |     test      | {email:"test@test.com"} | 2024-10-25 00:00:00.000000 | 2024-11-11 09:15:26.978272 |

- Token table

    |     user      |         token         |                  exp_time                  |
    | :-----------: | :-------------------: | :----------------------------------------: |
    | varchar - str |     varchar - str     |                datetime/int                |
    |     test      | ahsodhajkshdkanshdjka | 2024-10-25 00:00:00.000000/UNIX_TIME_STAMP |

- Group table

    |  id   |     name      | is_enable | user_id_list |        date_update         |
    | :---: | :-----------: | :-------: | :----------: | :------------------------: |
    |  int  | varchar - str |  tinyint  |     JSON     |          datetime          |
    |   0   |      dev      |     0     |  [1,2,3,4]   | 2024-10-25 00:00:00.000000 |

- Service table

    |  id   | service_name  | service_point | is_enable |        date_update         |
    | :---: | :-----------: | :-----------: | :-------: | :------------------------: |
    |  int  | varchar - str | varchar - str |  tinyint  |          datetime          |
    |   0   |     CMDB      | /an/api/route |     0     | 2024-10-25 00:00:00.000000 |

- Access table

    |  id   | service_id | access_id | group_access | is_enable |        date_update         |
    | :---: | :--------: | :-------: | :----------: | :-------: | :------------------------: |
    |  int  |    int     |    int    |   tinyint    |  tinyint  |          datetime          |
    |   0   |     0      |     0     |    accINT    |     0     | 2024-10-25 00:00:00.000000 |

- subsystem_table

    |  id   |  uuid   | service_name |              url              |   is_enable   |        date_update         | relate_service |
    | :---: | :-----: | :----------: | :---------------------------: | :-----------: | :------------------------: | :------------: |
    |  int  | varchar |   varchar    |            varchar            | varchar - str |          tinyint           |    datetime    | int |
    |   0   | XXXXXXX |   unnamed    | http://127.0.0.1:8000/api/hey |       0       | 2024-10-25 00:00:00.000000 |       0        |

## Deployment

### Config file

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
