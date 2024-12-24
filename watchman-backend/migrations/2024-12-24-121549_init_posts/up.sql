-- Your SQL goes here
-- 创建 access_table 表
DROP TABLE IF EXISTS `access_table`;

CREATE TABLE `access_table` (
    `id` INT UNSIGNED NOT NULL AUTO_INCREMENT COMMENT "access id",
    `service_id` INT UNSIGNED NOT NULL COMMENT "related service id",
    `group_id` INT UNSIGNED NOT NULL COMMENT "related group id",
    `group_access` TINYINT UNSIGNED NOT NULL COMMENT "access type",
    `is_enable` TINYINT UNSIGNED NOT NULL COMMENT "status",
    `update_time` DATETIME NULL,
    `comment` VARCHAR(255) NULL,
    PRIMARY KEY (`id`),
    KEY `service_id_key` (`service_id`),
    KEY `group_id_key` (`group_id`),
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci COMMENT "access table";

-- 创建 group_table 表
DROP TABLE IF EXISTS `group_table`;

CREATE TABLE `group_table` (
    `id` INT UNSIGNED NOT NULL AUTO_INCREMENT COMMENT "group id",
    `name` VARCHAR(255) NOT NULL COMMENT "group name",
    `is_enable` TINYINT UNSIGNED NOT NULL COMMENT "status",
    `date_update` DATETIME NULL,
    `user_ids` VARCHAR(255) NOT NULL COMMENT "users in the group JSON array",
    PRIMARY KEY (`id`),
    UNIQUE KEY `name_unique` (`name`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci COMMENT "group table";

-- 创建 service_table 表
DROP TABLE IF EXISTS `service_table`;

CREATE TABLE `service_table` (
    `id` INT UNSIGNED NOT NULL AUTO_INCREMENT COMMENT "service id",
    `service_name` VARCHAR(255) NOT NULL COMMENT "service name",
    `service_point` VARCHAR(255) NOT NULL COMMENT "service point route",
    `is_enable` TINYINT UNSIGNED NOT NULL COMMENT "status",
    `create_time` DATETIME NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `service_name_unique` (`service_name`),
    KEY `service_name_key` (`service_name`),
    KEY `service_point_key` (`service_point`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci COMMENT "service table";

-- 创建 subsystem_table 表
DROP TABLE IF EXISTS `subsystem_table`;

CREATE TABLE `subsystem_table` (
    `id` INT UNSIGNED NOT NULL AUTO_INCREMENT COMMENT "subsystem id",
    `subsys_name` VARCHAR(255) NOT NULL COMMENT "subsystem name",
    `url` VARCHAR(255) NOT NULL COMMENT "subsystem url",
    `is_enable` TINYINT UNSIGNED NOT NULL COMMENT "status",
    `update_time` DATETIME NULL,
    `relate_service` INT UNSIGNED NULL COMMENT "related service id",
    `token` VARCHAR(255) NOT NULL COMMENT "subsystem token (uuid)",
    PRIMARY KEY (`id`),
    UNIQUE KEY `subsys_name_unique` (`subsys_name`),
    KEY `subsys_name_key` (`subsys_name`),
    KEY `url_key` (`url`),
    KEY `relate_service_key` (`relate_service`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci COMMENT "subsystem table";

-- 创建 token_table 表
DROP TABLE IF EXISTS `token_table`;

CREATE TABLE `token_table` (
    `tokenid` VARCHAR(255) NOT NULL COMMENT "token id",
    `username` VARCHAR(255) NOT NULL COMMENT "username",
    `exp_time` TIMESTAMP NOT NULL COMMENT "token expire time",
    PRIMARY KEY (`tokenid`),
    UNIQUE KEY `username_unique` (`username`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci COMMENT "token table";

-- 创建 user_table 表
DROP TABLE IF EXISTS `user_table`;

CREATE TABLE `user_table` (
    `id` INT UNSIGNED NOT NULL AUTO_INCREMENT COMMENT "user id",
    `username` VARCHAR(255) NOT NULL COMMENT "username",
    `passwd` VARCHAR(255) NOT NULL COMMENT "password",
    `is_enable` TINYINT UNSIGNED NOT NULL COMMENT "status",
    `name` VARCHAR(255) NULL COMMENT "user's name",
    `contact` VARCHAR(255) NULL COMMENT "user contact information",
    `date_joined` DATETIME NULL COMMENT "user join time",
    `last_login` DATETIME NULL COMMENT "user last login time",
    PRIMARY KEY (`id`),
    UNIQUE KEY `username_unique` (`username`),
    KEY `username_key` (`username`),
    KEY `name_key` (`name`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci COMMENT "user table";