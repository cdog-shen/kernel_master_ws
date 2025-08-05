-- Your SQL goes here
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