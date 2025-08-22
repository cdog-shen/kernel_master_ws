# 文件代理

为整套系统提供一个文件交互能力.

具有上传/下载/ls等功能

***目前针对这个子系统的UUID auth逻辑还没有完成, 可以说目前只是一个测试用例(相对完整), 尽量不要部署到危险的网络环境中***

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
authenticate_bypass = ["/api/refresh_master"]
root = "./"
file_size_limit = 0

```