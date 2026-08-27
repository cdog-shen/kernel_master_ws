-- 重建通知组表（历史迁移中从未建过这两张表，此处按 schema.rs 中的 diesel 定义重建）

-- 通知组表
CREATE TABLE notification_groups (
    id SERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL,                 -- 通知组名称
    description VARCHAR(512),                   -- 通知组描述
    is_enabled BOOLEAN DEFAULT true,            -- 是否启用
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 通知组成员表
CREATE TABLE notification_group_members (
    id SERIAL PRIMARY KEY,
    group_id INTEGER NOT NULL REFERENCES notification_groups(id) ON DELETE CASCADE, -- 所属通知组
    channel_type VARCHAR(32) NOT NULL,          -- 通道类型: smtp/wecom/lark/dingtalk/ntfy/bark/teams
    recipient VARCHAR(512) NOT NULL,            -- 接收者(邮箱/用户ID/群组ID/openID/token等)
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_notification_group_members_group ON notification_group_members(group_id);
