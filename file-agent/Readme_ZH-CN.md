# 文件代理

为整套系统提供一个文件交互能力.

具有上传/下载/ls等功能

本子系统已接入 share-lib 统一鉴权中间件 (`UserAuth`): 请求可携带 `Authorization: Bearer <用户JWT>` (回源 watchman 鉴权) 或子系统 UUID (watchman 转发旧链路) 通过认证.

## 数据库结构

执行器不直接连接到数据库。

## 部署

- 依赖服务

    - 文件系统权限

### 配置文件

配置文件的名称是 `file_agent.toml`。

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