-- Your SQL goes here
-- KM系统表 --
-- 创建 cloud_account 表
DROP TABLE IF EXISTS cloud_account;

CREATE TABLE cloud_account (
    id SERIAL PRIMARY KEY,
    provider VARCHAR(255) NOT NULL,
    nick_name VARCHAR(255) NOT NULL,
    ak VARCHAR(255) NOT NULL,
    sk VARCHAR(255) NOT NULL,
    is_enable BOOLEAN NOT NULL,
    update_time TIMESTAMP NOT NULL,
    comment VARCHAR(255) NOT NULL
);

CREATE INDEX provider_idx ON cloud_account (provider);

CREATE INDEX nick_name_idx ON cloud_account (nick_name);

-- create job_log table
DROP TABLE IF EXISTS job_log;

CREATE TABLE job_log (
    id VARCHAR(255) PRIMARY KEY,
    script VARCHAR(255) NOT NULL,
    exec_type VARCHAR(255) NOT NULL,
    commander VARCHAR(255) NOT NULL,
    worker VARCHAR(255) NOT NULL,
    status SMALLINT NOT NULL CHECK (status BETWEEN 0 AND 255),
    params JSONB NOT NULL,
    result TEXT NOT NULL,
    finish_time TIMESTAMP NOT NULL,
    update_time TIMESTAMP NOT NULL,
    comment VARCHAR(255) NOT NULL
);

CREATE INDEX idx_job_log_exec_type ON job_log (exec_type);

CREATE INDEX idx_job_log_worker ON job_log (worker);

-- create cron_job table
DROP TABLE IF EXISTS cron_job;

CREATE TABLE cron_job (
    id VARCHAR(255) PRIMARY KEY,
    script VARCHAR(255) NOT NULL,
    frequency BIGINT NOT NULL,
    times BIGINT NOT NULL CHECK (times >= 0),
    params JSONB NOT NULL,
    comment VARCHAR(255) NOT NULL,
    is_enable BOOLEAN NOT NULL,
    launch_at TIMESTAMP NOT NULL,
    update_time TIMESTAMP NOT NULL
);

-- 业务表 --
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