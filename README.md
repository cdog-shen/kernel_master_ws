# kernel_master_ws

🎓 The workspace for my persional project kernel master (now it just a graduation project)

## Acknowledgements

🙏 Thank the user [Jami](https://github.com/jami1024) for providing the system architecture ideas.

🙏 Thank the user [SakaDream](https://github.com/SakaDream) for providing coding ideas on the project [actix-web-rest-api-with-jwt](https://github.com/SakaDream/actix-web-rest-api-with-jwt).

## share-lib : the public utils for all service

📚 [share-lib readme file](./share-lib/Readme.md)

## watchman-backend : the IAM and dispatch service for all subsystem

🆔 [Watch man readme file](./watchman-backend/Readme.md)

## Subsystems

Subsystem link to `watchman` service, called by `POST`ing `watchman/api/subsystem_call/{subsystem_name}/{operate}` from `watchman`

### CMDB: Configuration Management Database for all instance

🗂️ [CMDB readme file](./cmdb-backend/Readme.md)
