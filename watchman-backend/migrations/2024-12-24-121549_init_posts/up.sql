-- Your SQL goes here
-- 创建 access_table 表
DROP TABLE IF EXISTS access_table;

CREATE TABLE access_table (
    id SERIAL PRIMARY KEY CHECK (id >= 0),
    service_id INTEGER NOT NULL CHECK (service_id >= 0),
    group_id INTEGER NOT NULL CHECK (group_id >= 0),
    group_access SMALLINT NOT NULL CHECK (group_access BETWEEN 0 AND 5),
    is_enable BOOLEAN NOT NULL,
    update_time TIMESTAMP NOT NULL,
    comment VARCHAR(255) NOT NULL
);

CREATE INDEX access_table_service_id_idx ON access_table (service_id);

CREATE INDEX access_table_group_id_idx ON access_table (group_id);

INSERT INTO
    access_table (
        id,
        service_id,
        group_id,
        group_access,
        is_enable,
        update_time,
        comment
    )
VALUES (
        1,
        1,
        1,
        2,
        true,
        '2024-12-01 00:00:00',
        'auth:admin'
    ),
    (
        2,
        2,
        1,
        2,
        true,
        '2024-12-01 00:00:00',
        'group:admin'
    ),
    (
        3,
        3,
        1,
        2,
        true,
        '2024-12-01 00:00:00',
        'service:admin'
    ),
    (
        4,
        4,
        1,
        2,
        true,
        '2024-12-01 00:00:00',
        'access:admin'
    ),
    (
        5,
        5,
        1,
        2,
        true,
        '2024-12-01 00:00:00',
        'subsys:admin'
    );

-- 创建 group_table 表
DROP TABLE IF EXISTS group_table;

CREATE TABLE group_table (
    id SERIAL PRIMARY KEY CHECK (id >= 0),
    group_name VARCHAR(255) NOT NULL UNIQUE,
    is_enable BOOLEAN NOT NULL,
    user_ids JSONB NOT NULL,
    update_time TIMESTAMP NOT NULL
);

INSERT INTO
    group_table (
        id,
        group_name,
        is_enable,
        update_time,
        user_ids
    )
VALUES (
        1,
        'admin',
        true,
        '2024-12-01 00:00:00',
        '[1,2]'::jsonb
    ),
    (
        2,
        'script_caller',
        true,
        '2024-12-01 00:00:00',
        '[2]'::jsonb
    );

-- 创建 service_table 表
DROP TABLE IF EXISTS service_table;

CREATE TABLE service_table (
    id SERIAL PRIMARY KEY CHECK (id >= 0),
    service_name VARCHAR(255) NOT NULL UNIQUE,
    nick_name VARCHAR(255) NOT NULL,
    service_point VARCHAR(255) NOT NULL,
    is_enable BOOLEAN NOT NULL,
    update_time TIMESTAMP NOT NULL
);

CREATE INDEX service_table_service_name_idx ON service_table (service_name);

CREATE INDEX service_table_service_point_idx ON service_table (service_point);

INSERT INTO
    service_table (
        id,
        service_name,
        nick_name,
        service_point,
        is_enable,
        update_time
    )
VALUES (
        1,
        'auth',
        '认证服务',
        '/api/auth',
        true,
        '2024-12-01 00:00:00'
    ),
    (
        2,
        'group',
        '用户组控制',
        '/api/group',
        true,
        '2024-12-01 00:00:00'
    ),
    (
        3,
        'service',
        '服务控制',
        '/api/service',
        true,
        '2024-12-01 00:00:00'
    ),
    (
        4,
        'access',
        '权限控制',
        '/api/access',
        true,
        '2024-12-01 00:00:00'
    ),
    (
        5,
        'subsys',
        '子系统控制',
        '/api/subsystem',
        true,
        '2024-12-01 00:00:00'
    );

-- 创建 subsystem_table 表
DROP TABLE IF EXISTS subsystem_table;

CREATE TABLE subsystem_table (
    id SERIAL PRIMARY KEY CHECK (id >= 0),
    subsys_name VARCHAR(255) NOT NULL UNIQUE,
    url VARCHAR(255) NOT NULL,
    is_enable BOOLEAN NOT NULL,
    relate_service_id INTEGER NOT NULL CHECK (relate_service_id >= 0),
    token UUID NOT NULL,
    update_time TIMESTAMP NOT NULL
);

CREATE INDEX subsystem_table_subsys_name_idx ON subsystem_table (subsys_name);

CREATE INDEX subsystem_table_url_idx ON subsystem_table (url);

CREATE INDEX subsystem_table_relate_service_idx ON subsystem_table (relate_service_id);

-- 创建 token_table 表
DROP TABLE IF EXISTS token_table;

CREATE TABLE token_table (
    tokenid VARCHAR(255) PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    exp_time TIMESTAMP NOT NULL
);

-- 创建 user_table 表
DROP TABLE IF EXISTS user_table;

CREATE TABLE user_table (
    id SERIAL PRIMARY KEY CHECK (id >= 0),
    username VARCHAR(255) NOT NULL UNIQUE,
    passwd VARCHAR(255) NOT NULL,
    is_enable BOOLEAN NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    contact JSONB NOT NULL,
    last_login TIMESTAMP NOT NULL,
    update_time TIMESTAMP NOT NULL
);

CREATE INDEX user_table_username_idx ON user_table (username);

CREATE INDEX user_table_full_name_idx ON user_table (full_name);

INSERT INTO
    user_table (
        id,
        username,
        passwd,
        is_enable,
        full_name,
        contact,
        update_time,
        last_login
    )
VALUES (
        1,
        'root',
        'root',
        true,
        'root as admin',
        '{}'::jsonb,
        '2024-12-01 00:00:00',
        '2024-12-01 00:00:00'
    ),
    (
        2,
        'script_caller',
        'script_caller',
        true,
        'subsystem user for calling script',
        '{}'::jsonb,
        '2024-12-01 00:00:00',
        '2024-12-01 00:00:00'
    );