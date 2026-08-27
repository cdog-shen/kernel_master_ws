-- 删除未使用的通知组死表
-- 先删成员表（其 group_id 外键依赖 notification_groups）
DROP TABLE IF EXISTS notification_group_members;
DROP TABLE IF EXISTS notification_groups;
