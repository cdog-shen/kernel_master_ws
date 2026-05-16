-- 跨子系统表：watchman-backend 表
-- 以下表结构与 watchman-backend 保持一致

-- access_table
CREATE TABLE IF NOT EXISTS access_table (
    id SERIAL PRIMARY KEY CHECK (id >= 0),
    service_id INTEGER NOT NULL CHECK (service_id >= 0),
    group_id INTEGER NOT NULL CHECK (group_id >= 0),
    group_access SMALLINT NOT NULL CHECK (group_access BETWEEN 0 AND 5),
    is_enable BOOLEAN NOT NULL,
    update_time TIMESTAMP NOT NULL,
    comment VARCHAR(255) NOT NULL
);

CREATE INDEX IF NOT EXISTS access_table_service_id_idx ON access_table (service_id);
CREATE INDEX IF NOT EXISTS access_table_group_id_idx ON access_table (group_id);

-- group_table
CREATE TABLE IF NOT EXISTS group_table (
    id SERIAL PRIMARY KEY CHECK (id >= 0),
    group_name VARCHAR(255) NOT NULL UNIQUE,
    is_enable BOOLEAN NOT NULL,
    user_ids JSONB NOT NULL,
    update_time TIMESTAMP NOT NULL
);

-- service_table
CREATE TABLE IF NOT EXISTS service_table (
    id SERIAL PRIMARY KEY CHECK (id >= 0),
    service_name VARCHAR(255) NOT NULL UNIQUE,
    nick_name VARCHAR(255) NOT NULL,
    service_point VARCHAR(255) NOT NULL,
    is_enable BOOLEAN NOT NULL,
    update_time TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS service_table_service_name_idx ON service_table (service_name);
CREATE INDEX IF NOT EXISTS service_table_service_point_idx ON service_table (service_point);

-- subsystem_table
CREATE TABLE IF NOT EXISTS subsystem_table (
    id SERIAL PRIMARY KEY CHECK (id >= 0),
    subsys_name VARCHAR(255) NOT NULL UNIQUE,
    url VARCHAR(255) NOT NULL,
    is_enable BOOLEAN NOT NULL,
    relate_service_id INTEGER NOT NULL CHECK (relate_service_id >= 0),
    token UUID NOT NULL,
    update_time TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS subsystem_table_subsys_name_idx ON subsystem_table (subsys_name);
CREATE INDEX IF NOT EXISTS subsystem_table_url_idx ON subsystem_table (url);
CREATE INDEX IF NOT EXISTS subsystem_table_relate_service_idx ON subsystem_table (relate_service_id);

-- token_table
CREATE TABLE IF NOT EXISTS token_table (
    tokenid VARCHAR(255) PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    exp_time TIMESTAMP NOT NULL
);

-- user_table
CREATE TABLE IF NOT EXISTS user_table (
    id SERIAL PRIMARY KEY CHECK (id >= 0),
    username VARCHAR(255) NOT NULL UNIQUE,
    passwd VARCHAR(255) NOT NULL,
    is_enable BOOLEAN NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    contact JSONB NOT NULL,
    last_login TIMESTAMP NOT NULL,
    update_time TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS user_table_username_idx ON user_table (username);
CREATE INDEX IF NOT EXISTS user_table_full_name_idx ON user_table (full_name);

-- webhook_table
CREATE TABLE IF NOT EXISTS webhook_table (
    id SERIAL PRIMARY KEY CHECK (id >= 0),
    token VARCHAR(255) NOT NULL UNIQUE,
    hook_name VARCHAR(255) NOT NULL UNIQUE,
    method_type VARCHAR(255) NOT NULL,
    target_url VARCHAR(255) NOT NULL,
    query_json JSONB NOT NULL,
    header_json JSONB NOT NULL,
    body_json JSONB NOT NULL,
    ttl BIGINT NOT NULL,
    is_enable BOOLEAN NOT NULL,
    update_time TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS webhook_table_username_idx ON webhook_table (hook_name);
CREATE INDEX IF NOT EXISTS webhook_table_full_name_idx ON webhook_table (token);

-- 跨子系统表：yell 表
-- 以下表结构与 yell 保持一致

-- channel_configs
CREATE TABLE IF NOT EXISTS channel_configs (
    id SERIAL PRIMARY KEY,
    channel_type VARCHAR(32) NOT NULL UNIQUE,
    config_json JSONB NOT NULL,
    is_enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- notification_records
CREATE TABLE IF NOT EXISTS notification_records (
    id SERIAL PRIMARY KEY,
    channel_type VARCHAR(32) NOT NULL,
    recipient VARCHAR(512) NOT NULL,
    subject VARCHAR(512),
    content TEXT NOT NULL,
    content_format VARCHAR(16) DEFAULT 'text',
    template_id INTEGER,
    status VARCHAR(16) DEFAULT 'pending',
    error_msg TEXT,
    sent_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_notification_records_channel ON notification_records (channel_type);
CREATE INDEX IF NOT EXISTS idx_notification_records_status ON notification_records (status);
CREATE INDEX IF NOT EXISTS idx_notification_records_created_at ON notification_records (created_at);

-- notification_templates
CREATE TABLE IF NOT EXISTS notification_templates (
    id SERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL UNIQUE,
    description VARCHAR(512),
    channel_type VARCHAR(32) NOT NULL,
    subject_template VARCHAR(512),
    content_template TEXT NOT NULL,
    content_format VARCHAR(16) DEFAULT 'text',
    is_enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- notification_groups
CREATE TABLE IF NOT EXISTS notification_groups (
    id SERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL,
    description VARCHAR(512),
    is_enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- notification_group_members
CREATE TABLE IF NOT EXISTS notification_group_members (
    id SERIAL PRIMARY KEY,
    group_id INTEGER NOT NULL REFERENCES notification_groups(id),
    channel_type VARCHAR(32) NOT NULL,
    recipient VARCHAR(512) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
