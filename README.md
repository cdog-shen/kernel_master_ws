# kernel_master_ws

🎓 The workspace for my persional project kernel master (now it just a graduation project)

## Acknowledgements

🙏 Thank the user [SakaDream](https://github.com/SakaDream) for providing coding ideas on the project [actix-web-rest-api-with-jwt](https://github.com/SakaDream/actix-web-rest-api-with-jwt).

## share-lib : the public utils for all service

📚 [share-lib readme file](./share-lib/Readme.md)

## watchman-backend : the IAM and dispatch service for all subsystem

🆔 [Watch man readme file](./watchman-backend/Readme.md)

### AuthDB

- User table

    |id|user|passwd|is_enable|name|contact|date_joined|last_login|
    |:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
    |uint|varchar - str|varchar - str|tinyint|varchar - str|JSON|datetime|datetime|
    | 0 |testuser| 00000000 | 1 | test| {email:"test@test.com"} |2024-10-25 00:00:00.000000|2024-11-11 09:15:26.978272|

- Token table

    |user|token|exp_time|
    |:-:|:-:|:-:|
    |varchar - str|varchar - str|datetime/int|
    |test|ahsodhajkshdkanshdjka|2024-10-25 00:00:00.000000/UNIX_TIME_STAMP|

- Group table

    |id|name|is_enable|user_id_list|date_update|
    |:-:|:-:|:-:|:-:|:-:|
    |int|varchar - str|tinyint|JSON|datetime|
    |0|dev|0|[1,2,3,4]|2024-10-25 00:00:00.000000|

- Service table

    |id|service_name|service_point|is_enable|date_update|
    |:-:|:-:|:-:|:-:|:-:|
    |int|varchar - str|varchar - str|tinyint|datetime|
    |0|CMDB|/an/api/route|0|2024-10-25 00:00:00.000000|

- Access table

    |id|service_id|access_id|group_access|is_enable|date_update|
    |:-:|:-:|:-:|:-:|:-:|:-:|
    |int|int|int|tinyint|varchar - str|tinyint|datetime|
    |0|0|0|accINT|0|2024-10-25 00:00:00.000000|

- subsystem_table

    |id|uuid|service_name|url|is_enable|date_update|
    |:-:|:-:|:-:|:-:|:-:|:-:|
    |int|varchar|varchar|varchar|varchar - str|tinyint|datetime|
    |0|XXXXXXX|unnamed|http://127.0.0.1:8000/api/hey|0|2024-10-25 00:00:00.000000|

## CMDB