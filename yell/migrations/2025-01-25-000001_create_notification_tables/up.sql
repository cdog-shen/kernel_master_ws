-- 通知记录表
CREATE TABLE notification_records (
    id SERIAL PRIMARY KEY,
    channel_type VARCHAR(32) NOT NULL,          -- 通道类型: smtp/wecom/lark/dingtalk/ntfy/bark/teams
    recipient VARCHAR(512) NOT NULL,            -- 接收者(邮箱/用户ID/群组ID/openID/token等)
    subject VARCHAR(512),                       -- 主题/标题
    content TEXT NOT NULL,                      -- 内容
    content_format VARCHAR(16) DEFAULT 'text',  -- 格式: text/html/markdown
    template_id INTEGER,                        -- 使用的模板ID(可选)
    status VARCHAR(16) DEFAULT 'pending',       -- 状态: pending/sent/failed
    error_msg TEXT,                             -- 错误信息
    sent_at TIMESTAMP,                          -- 发送时间
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 通知模板表
CREATE TABLE notification_templates (
    id SERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL UNIQUE,          -- 模板名称
    description VARCHAR(512),                   -- 模板描述
    channel_type VARCHAR(32) NOT NULL,          -- 适用通道类型
    subject_template VARCHAR(512),              -- 主题模板
    content_template TEXT NOT NULL,             -- 内容模板
    content_format VARCHAR(16) DEFAULT 'text',  -- 格式: text/html/markdown
    is_enabled BOOLEAN DEFAULT true,            -- 是否启用
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 通道配置表(存储 SMTP/企业微信等配置)
CREATE TABLE channel_configs (
    id SERIAL PRIMARY KEY,
    channel_type VARCHAR(32) NOT NULL UNIQUE,   -- 通道类型
    config_json JSONB NOT NULL,                 -- 配置JSON, 各渠道自行定义结构
    is_enabled BOOLEAN DEFAULT true,            -- 是否启用
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引
CREATE INDEX idx_notification_records_channel ON notification_records(channel_type);
CREATE INDEX idx_notification_records_status ON notification_records(status);
CREATE INDEX idx_notification_records_created_at ON notification_records(created_at);

-- 插入默认 SMTP 配置模板
INSERT INTO channel_configs (channel_type, config_json, is_enabled) VALUES (
    'smtp',
    '{
        "host": "localhost",
        "port": 587,
        "username": "",
        "password": "",
        "from": "",
        "use_tls": true
    }'::jsonb,
    false
);
