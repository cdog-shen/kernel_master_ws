-- Your SQL goes here
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