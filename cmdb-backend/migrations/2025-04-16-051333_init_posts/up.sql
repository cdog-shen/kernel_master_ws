-- Your SQL goes here
-- 创建 lighthouse_instance 表
DROP TABLE IF EXISTS lighthouse_instance;

CREATE TABLE lighthouse_instance (
    id BIGSERIAL PRIMARY KEY,
    provider VARCHAR(255) NOT NULL,
    zone VARCHAR(255) NOT NULL,
    instance_id VARCHAR(255) NOT NULL,
    instance_name VARCHAR(255) NOT NULL,
    platform VARCHAR(255) NOT NULL,
    status VARCHAR(255) NOT NULL,
    private_ip VARCHAR(255) NOT NULL,
    tag JSONB NOT NULL,
    full_info JSONB NOT NULL,
    attach_info JSONB NOT NULL,
    update_at TIMESTAMP NOT NULL
);

-- 创建 cloudserver_instance 表
DROP TABLE IF EXISTS cloudserver_instance;

CREATE TABLE cloudserver_instance (
    id BIGSERIAL PRIMARY KEY,
    provider VARCHAR(255) NOT NULL,
    zone VARCHAR(255) NOT NULL,
    instance_id VARCHAR(255) NOT NULL,
    instance_name VARCHAR(255) NOT NULL,
    platform VARCHAR(255) NOT NULL,
    status VARCHAR(255) NOT NULL,
    private_ip VARCHAR(255) NOT NULL,
    tag JSONB NOT NULL,
    full_info JSONB NOT NULL,
    attach_info JSONB NOT NULL,
    update_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_cloudserver_provider ON cloudserver_instance (provider);

CREATE INDEX idx_cloudserver_instance_id ON cloudserver_instance (instance_id);

CREATE INDEX idx_cloudserver_instance_name ON cloudserver_instance (instance_name);

CREATE INDEX idx_cloudserver_zone ON cloudserver_instance (zone);

-- 创建 logservice_topic 表
DROP TABLE IF EXISTS logservice_topic;

CREATE TABLE logservice_topic (
    id BIGSERIAL PRIMARY KEY,
    provider VARCHAR(255) NOT NULL,
    set_id VARCHAR(255) NOT NULL,
    topic_id VARCHAR(255) NOT NULL,
    topic_name VARCHAR(255) NOT NULL,
    status VARCHAR(255) NOT NULL,
    hot_period INTEGER NOT NULL CHECK (hot_period >= 0),
    period INTEGER NOT NULL CHECK (period >= 0),
    index BOOLEAN NOT NULL,
    describes VARCHAR(255) NOT NULL,
    tag JSONB NOT NULL,
    full_info JSONB NOT NULL,
    attach_info JSONB NOT NULL,
    update_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_logservice_topic_provider ON logservice_topic (provider);

CREATE INDEX idx_logservice_topic_instance_id ON logservice_topic (topic_id);

CREATE INDEX idx_logservice_topic_instance_name ON logservice_topic (topic_name);