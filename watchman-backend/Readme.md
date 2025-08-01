# Watchman-backend

Watchman is the IAM (Identity & Access Management) and orchestration service for the entire Kernel Master project.  
It should be the **first** service to start among all Kernel Master components.

## Code Conventions

- All ORM-related operations must be placed in the corresponding module under the ***model*** directory.

    Input is mapped via `from map`; any missing fields are treated as `None`.

    Prefer using the `Info/Model` pair for input and output.  
    The `User` struct is the **only** exception because the `passwd` field must never be exposed.

    CRUD methods should only contain basic implementations.  
    When creating new methods, add field validation and default values as necessary.  
    These methods must accept `ref` parameters to avoid reuse difficulties in the **Service** layer.

- All interface handling logic belongs in the ***service*** directory.

    This layer contains the main business logic and accepts parameters **by value** (not `ref`).  
    It is responsible for calling logic in other modules.

- API-response–related logic goes into the corresponding API module under the ***api*** directory.

    All query endpoints must use `GET`, update endpoints `POST`, and delete endpoints `DELETE`.  
    This layer performs **no** business logic—only basic data processing—so the **Service** layer receives clean data.

## APIs

All endpoints are prefixed with `/api`.

### /hey

| Resource | Supported Methods | Purpose                                  | Notes              |
| :------: | :---------------: | :--------------------------------------- | :----------------- |
|    /     |   `POST`/`GET`    | Returns the raw string `hi hello!`       | Service health API |

### /reload

| Resource | Supported Methods | Purpose           | Notes                                   |
| :------: | :---------------: | :---------------- | :-------------------------------------- |
|    /     |      `POST`       | Reloads all config | Hot-reloads dynamic configuration files |

### /auth

|   Resource    | Supported Methods | Purpose                      | Notes                                 |
| :-----------: | :---------------: | :--------------------------- | :------------------------------------ |
|  /all_user    |       `GET`       | Retrieve all users           |                                       |
|  /me/{id}     |       `GET`       | Retrieve a single user       | Returns full user profile             |
|  /signup      |      `POST`       | Register a new user          |                                       |
|  /login       |      `POST`       | Login with username/password |                                       |
|  /logout      |      `POST`       | Logout                       |                                       |
| /user_update  |      `POST`       | Update user information      |                                       |

### /group_control

|    Resource     | Supported Methods | Purpose             | Notes |
| :-------------: | :---------------: | :------------------ | :---- |
|  /all_group     |       `GET`       | List all groups     |       |
|  /new_group     |      `POST`       | Create a group      |       |
| /update_group   |      `POST`       | Update a group      |       |
| /delete_group   |     `DELETE`      | Delete a group      |       |

### /service_control

|     Resource      | Supported Methods | Purpose               | Notes                                     |
| :---------------: | :---------------: | :-------------------- | :---------------------------------------- |
|  /all_service     |       `GET`       | List all services     |                                           |
|  /new_service     |      `POST`       | Create a service      |                                           |
| /update_service   |      `POST`       | Update a service      |                                           |
| /delete_service   |     `DELETE`      | Delete a service      | Also disables any associated access rules |

### /access_control

|    Resource     | Supported Methods | Purpose             | Notes |
| :-------------: | :---------------: | :------------------ | :---- |
|  /all_access    |       `GET`       | List all access     |       |
|  /new_access    |      `POST`       | Create an access    |       |
| /update_access  |      `POST`       | Update an access    |       |
| /delete_access  |     `DELETE`      | Delete an access    |       |

### /subsystem_control

|      Resource       | Supported Methods | Purpose                       | Notes                                                                               |
| :-----------------: | :---------------: | :---------------------------- | :---------------------------------------------------------------------------------- |
| /all_subsystem      |       `GET`       | List all subsystems           |                                                                                     |
| /new_subsystem      |      `POST`       | Create a subsystem            | Also creates the bound service                                                      |
| /update_subsystem   |      `POST`       | Update a subsystem            |                                                                                     |
| /delete_subsystem   |     `DELETE`      | Delete a subsystem            | Also disables related access rules and deletes any bound services                   |

### /subsystem_call

|      Resource       | Supported Methods | Purpose                                 | Notes                                                                                         |
| :-----------------: | :---------------: | :-------------------------------------- | :-------------------------------------------------------------------------------------------- |
| /{subsystem_name}   |      `POST`       | Invoke a subsystem service via JSON     | If the subsystem does not return JSON, the response is wrapped as `{"data":"any data"}` |

## Database Schema

- **users**

    | id  | username | password | enabled | display_name | contact_info | created_at | last_login |
    |:---:|:--------:|:--------:|:-------:|:------------:|:------------:|:----------:|:----------:|
    | uint| varchar  | varchar  | tinyint | varchar      | JSON         | datetime   | datetime   |
    | 0   | testuser | 00000000 | 1       | test         | {"email":"test@test.com"} | 2024-10-25 00:00:00.000 | 2024-11-11 09:15:26.978 |

- **tokens**

    | username | token                 | expires_at                  |
    |:--------:|:---------------------:|:---------------------------:|
    | varchar  | varchar               | datetime / UNIX_TIME_STAMP  |
    | test     | ahsodhajkshdkanshdjka | 2024-10-25 00:00:00.000     |

- **groups**

    | id  | name    | enabled | user_ids | updated_at |
    |:---:|:--------|:-------:|:--------:|:----------:|
    | int | varchar | tinyint | JSON     | datetime   |
    | 0   | dev     | 0       | [1,2,3,4]| 2024-10-25 00:00:00.000 |

- **services**

    | id  | name    | endpoint     | enabled | updated_at |
    |:---:|:--------|:-------------|:-------:|:----------:|
    | int | varchar | varchar      | tinyint | datetime   |
    | 0   | CMDB    | /an/api/route| 0       | 2024-10-25 00:00:00.000 |

- **access_rules**

    | id  | service_id | access_id | group_access | enabled | updated_at |
    |:---:|:----------:|:---------:|:------------:|:-------:|:----------:|
    | int | int        | int       | tinyint      | tinyint | datetime   |
    | 0   | 0          | 0         | accINT       | 0       | 2024-10-25 00:00:00.000 |

- **subsystems**

    | id  | uuid    | name    | url                      | enabled | updated_at | related_service |
    |:---:|:--------|:--------|:-------------------------|:-------:|:----------:|:---------------:|
    | int | varchar | varchar | http://127.0.0.1:8000/api/hey | tinyint | datetime   | int             |
    | 0   | XXXXXXX | unnamed | http://127.0.0.1:8000/api/hey | 0       | 2024-10-25 00:00:00.000 | 0               |

## Deployment & Dependencies

- **Key Generation**

    Before starting the service, generate a random key for JWT signing (optional; the system will auto-generate one if absent).

    ```sh
    openssl rand -hex 32 > key/jwt_secret.key
    ```

- **System Libraries**

    - openssl
    - libpq

- **Required Services**

    - PostgreSQL

### Configuration File

Create `watchman.toml` in the project root:

```toml
# Server configuration
[server_config]
# Logging
log_path = "log/watchman.log"
log_level = "DEBUG"
clear_log = true
# Server params
listen_addr = "0.0.0.0"
listen_port = 8000
workers = 2
# CORS
allowed_origin_list = [
    "http://localhost:3000",
    "http://127.0.0.1:3000"
]
# JWT secret
secret_key_path = "key/jwt_secret.key"
# Auth & permit bypass lists
authenticate_bypass = [
    "/api/hey",
    "/webhook",
    "/api/reload",
    "/api/auth/login",
    "/api/auth/signup",
    "/api/subsystem_control/all_subsystem",
    "/api/subsystem_control/update_subsystem",
]
permit_bypass = [
    "/api/hey",
    "/webhook",
    "/api/reload",
    "/api/auth/me",
    "/api/auth/login",
    "/api/auth/signup",
    "/api/subsystem_control/all_subsystem",
    "/api/subsystem_control/update_subsystem",
]

# Database
[db_config]
db_str = "postgres://postgres:password@localhost:5432/watch_man"
```

# Q & A

- **Q:** Under high load, RPC calls to subsystems occasionally hang.

  **A:** Simply increase the number of worker threads; this will **not** raise overall CPU load.