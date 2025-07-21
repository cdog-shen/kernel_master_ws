
<div align="center">
  <p>
    <a href="./README_ZH-CN.md">🇨🇳 简体中文</a> | 
    <a href="./README.md">🇺🇸/🇬🇧 English</a> | 
  </p>
</div>

# About

🎓 The workspace for my personal project kernel master (now it just a graduation project)

## Acknowledgements

🙏 Thank the user [Jami](https://github.com/jami1024) for providing the system architecture ideas.

🙏 Thank the user [SakaDream](https://github.com/SakaDream) for providing coding ideas on the project [actix-web-rest-api-with-jwt](https://github.com/SakaDream/actix-web-rest-api-with-jwt).

## share-lib : the public utils for all service

📚 [share-lib readme file](./share-lib/Readme.md)

## watchman-backend : the IAM and dispatch service for all subsystem

🆔 [Watch man readme file](./watchman-backend/Readme.md)

## Subsystems

Subsystem link to `watchman` service

Called by `POST` to `watchman/api/subsystem_call/{subsystem_name}/{operate}` from `watchman`

### CMDB: Configuration Management Database for all instance

🗂️ [CMDB readme file](./cmdb-backend/Readme.md)

### Cloud-API: A unified interface subsystem for cloud asset rights management and invocation

☁️ [Cloud-API readme file](./cloud-api/Readme.md)

### Job-Center: A subsystem for executing synchronous/asynchronous scripts

This subsystem is divided into two parts: the worker and the commander.

📡 [controller readme file](./jc-commander/Readme.md)

📋 [worker readme file](./jc-worker/Readme.md)

## DockerFile

🐳 Dockerfile is used to build an `Ubuntu` container, with some built-in static dependencies and variables for static compilation, and can be compiled with `build/build_all_ws.sh` if necessary.

U can build project by using (target: `x86_64-unknown-linux-gnu`):

```sh
docker-compose --profile build up
```

And launch it by using:

```sh
# Before execute, Make your message queue and database ready
# Don't forget to change config file 
docker-compose --profile run up -d
```
