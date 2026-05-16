-- 回滚：移除 name 字段，恢复 channel_type 唯一约束

-- 1. 删除联合唯一约束
ALTER TABLE channel_configs
    DROP CONSTRAINT IF EXISTS channel_configs_channel_name_unique;

-- 2. 删除 name 列
ALTER TABLE channel_configs
    DROP COLUMN IF EXISTS name;

-- 3. 恢复 channel_type 唯一约束
ALTER TABLE channel_configs
    ADD CONSTRAINT channel_configs_channel_type_key UNIQUE (channel_type);
