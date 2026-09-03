# File Agent

Provides the whole system with file-interaction capabilities.  
Supports upload / download / ls, etc.

This subsystem is protected by the shared auth middleware (`share-lib` `UserAuth`): requests authenticate either via `Authorization: Bearer <user JWT>` (verified against watchman) or via the subsystem UUID (legacy watchman-forwarded path).

## Database Structure
The executor does **not** connect to the database directly.

## Deployment

- Dependencies
    - File-system permissions

### Configuration File

Configuration file name: `file_agent.toml`

```toml
[server_config]
log_path = "log/fileagent.log"
log_level = "DEBUG"
clear_log = "True"
listen_addr = "0.0.0.0"
listen_port = 9004
register_name = "file-agent"
master_addr = "127.0.0.1"
master_port = 8000
workers = 8
allowed_origin_list = ["http://localhost:3000", "http://127.0.0.1:3000"]
authenticate_bypass = ["/api/hey", "/api/manage/refresh_master"]
root = "./"
file_size_limit = 0
```