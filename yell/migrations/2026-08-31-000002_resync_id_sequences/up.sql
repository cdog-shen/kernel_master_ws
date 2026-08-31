-- 重新同步各表 serial 序列
-- 背景：存量数据曾带显式 id 写入（种子数据/数据导入），序列落后于 max(id)，
-- 自增主键与存量行冲突（duplicate key value violates unique constraint "*_pkey"）
-- 处理：空表回退为 (1, false) 使下一个 id 从 1 开始；非空表对齐到 max(id)

SELECT setval('notification_records_id_seq', GREATEST(m, 1), m > 0)
FROM (SELECT COALESCE(MAX(id), 0) AS m FROM notification_records) t;

SELECT setval('notification_aliases_id_seq', GREATEST(m, 1), m > 0)
FROM (SELECT COALESCE(MAX(id), 0) AS m FROM notification_aliases) t;

SELECT setval('notification_templates_id_seq', GREATEST(m, 1), m > 0)
FROM (SELECT COALESCE(MAX(id), 0) AS m FROM notification_templates) t;

SELECT setval('channel_configs_id_seq', GREATEST(m, 1), m > 0)
FROM (SELECT COALESCE(MAX(id), 0) AS m FROM channel_configs) t;
