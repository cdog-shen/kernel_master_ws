<div align="center">
    <p>
        <a href="./README_ZH-CN.md">🇨🇳 简体中文</a> | 
        <a href="./README.md">🇺🇸/🇬🇧 English</a> | 
    </p>
</div>

# 关于

🎓 这是我个人项目 kernel master 的工作区（现在只是一个毕业项目）

## 致谢

🙏 感谢用户 [Jami](https://github.com/jami1024) 提供的系统架构思路。

🙏 感谢用户 [SakaDream](https://github.com/SakaDream) 在项目 [actix-web-rest-api-with-jwt](https://github.com/SakaDream/actix-web-rest-api-with-jwt) 中提供的编码思路。

## share-lib : 所有服务的公共工具库

📚 [share-lib 说明文件](./share-lib/Readme_ZH-CN.md)

## watchman-backend : 所有子系统的 IAM 和调度服务

🆔 [Watchman 说明文件](./watchman-backend/Readme_ZH-CN.md)

## 子系统

子系统链接到 `watchman` 服务

通过 `POST` 请求 `watchman/api/subsystem_call/{subsystem_name}/{operate}` 从 `watchman` 调用

### CMDB: 所有实例的配置管理数据库

🗂️ [CMDB 说明文件](./cmdb-backend/Readme_ZH-CN.md)

### Cloud-API: 用于云资产权限管理和调用的统一接口子系统

☁️ [Cloud-API 说明文件](./cloud-api/Readme_ZH-CN.md)

### Job-Center: 用于执行同步/异步脚本的子系统

该子系统分为两部分：worker 和 commander。

📡 [controller 说明文件](./jc-commander/Readme_ZH-CN.md)

📋 [worker 说明文件](./jc-worker/Readme_ZH-CN.md)

## DockerFile

🐳 Dockerfile用于构建一个 `alpine linux` 的容器, 内置了一些构建需要的静态依赖库和变量, 用于静态编译, 如果有需要可以使用 `build/build_all_ws_static.sh` 来进行编译

使用build profile来进行编译项目(目标: `x86_64-unknown-linux-gnu`):

```sh
docker-compose --profile build up
```

使用下面的命令启动整个服务:

```sh
# 开始之前, 确保你的MQ和DB是可用的状态
# 别忘了修改配置文件
docker-compose --profile run up -d
```
