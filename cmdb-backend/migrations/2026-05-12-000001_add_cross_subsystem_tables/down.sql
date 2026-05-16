-- Undo cross-subsystem tables

DROP TABLE IF EXISTS notification_group_members;
DROP TABLE IF EXISTS notification_groups;
DROP TABLE IF EXISTS notification_templates;
DROP TABLE IF EXISTS notification_records;
DROP TABLE IF EXISTS channel_configs;

DROP TABLE IF EXISTS webhook_table;
DROP TABLE IF EXISTS user_table;
DROP TABLE IF EXISTS token_table;
DROP TABLE IF EXISTS subsystem_table;
DROP TABLE IF EXISTS service_table;
DROP TABLE IF EXISTS group_table;
DROP TABLE IF EXISTS access_table;
