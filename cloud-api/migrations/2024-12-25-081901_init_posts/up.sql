-- Your SQL goes here
-- 创建 cloud_account 表
DROP TABLE IF EXISTS `cloud_account`;

CREATE TABLE `cloud_account` (
    `id` INT UNSIGNED NOT NULL AUTO_INCREMENT COMMENT "access id",
    `cloud_provider` VARCHAR(255) NOT NULL COMMENT "cloud provider",
    `nick_name` VARCHAR(255) NOT NULL COMMENT "nick name",
    `ak` VARCHAR(255) NOT NULL COMMENT "access key",
    `sk` VARCHAR(255) NOT NULL COMMENT "secret",
    `is_enable` TINYINT UNSIGNED NOT NULL COMMENT "status",
    `update_time` DATETIME NULL,
    `comment` VARCHAR(255) NULL COMMENT "access comment",
    PRIMARY KEY (`id`),
    KEY `cloud_provider_key` (`cloud_provider`),
    KEY `nick_name_key` (`nick_name`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci;