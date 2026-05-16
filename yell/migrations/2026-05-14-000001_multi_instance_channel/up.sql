-- 为 channel_configs 表添加 name 字段，支持同一渠道多实例配置

-- 1. 添加 name 列（默认空字符串，作为默认配置）
ALTER TABLE channel_configs
    ADD COLUMN name VARCHAR(128) NOT NULL DEFAULT '';

-- 2. 删除旧的 channel_type 唯一约束
ALTER TABLE channel_configs
    DROP CONSTRAINT IF EXISTS channel_configs_channel_type_key;

-- 3. 添加 channel_type + name 联合唯一约束
ALTER TABLE channel_configs
    ADD CONSTRAINT channel_configs_channel_name_unique UNIQUE (channel_type, name);
