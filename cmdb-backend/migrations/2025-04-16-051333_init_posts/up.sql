-- Your SQL goes here
-- 创建 lighthouse_instance 表
DROP TABLE IF EXISTS `lighthouse_instance`;

CREATE TABLE `lighthouse_instance` (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT COMMENT "Primary key",
    `cloud_name` VARCHAR(255) NOT NULL COMMENT "Cloud provider name",
    `region` VARCHAR(255) NOT NULL COMMENT "Region",
    `zone` VARCHAR(255) NOT NULL COMMENT "Zone",
    `instance_id` VARCHAR(255) NOT NULL COMMENT "Instance ID",
    `instance_name` VARCHAR(255) NOT NULL COMMENT "Instance name",
    `wip` VARCHAR(255) NULL DEFAULT NULL COMMENT "WAN IP",
    `vpc_id` VARCHAR(255) NULL DEFAULT NULL COMMENT "VPC ID",
    `status` VARCHAR(255) NOT NULL COMMENT "Instance status",
    `os_name` VARCHAR(255) NOT NULL COMMENT "Operating system name",
    `os_type` VARCHAR(255) NOT NULL COMMENT "Operating system type",
    `create_at` TIMESTAMP NULL DEFAULT NULL COMMENT "Creation timestamp",
    `update_at` TIMESTAMP NULL DEFAULT NULL COMMENT "Update timestamp",
    `full_info` TEXT NULL DEFAULT NULL COMMENT "Full information",
    `attach_info` TEXT NULL DEFAULT NULL COMMENT "Attachment information",
    PRIMARY KEY (`id`),
    UNIQUE KEY `id` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci;

-- 创建 cloudserver_instance 表
DROP TABLE IF EXISTS `cloudserver_instance`;

CREATE TABLE `cloudserver_instance` (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT COMMENT "Primary key",
    `provider` VARCHAR(255) NOT NULL COMMENT "Cloud provider name",
    `zone` VARCHAR(255) NOT NULL COMMENT "Zone",
    `instance_id` VARCHAR(255) NOT NULL COMMENT "Instance ID",
    `instance_name` VARCHAR(255) NOT NULL COMMENT "Instance name",
    `plantform` VARCHAR(255) NOT NULL COMMENT "plantform",
    `status` VARCHAR(255) NOT NULL COMMENT "Instance status",
    `tag` VARCHAR(255) NOT NULL COMMENT "Instance Tag",
    `private_ip` VARCHAR(255) NOT NULL COMMENT "Instance VPC IP",
    `full_info` TEXT NULL DEFAULT NULL COMMENT "Full information",
    `attach_info` TEXT NULL DEFAULT NULL COMMENT "Attachment information",
    `update_at` TIMESTAMP NULL DEFAULT NULL COMMENT "Update timestamp",
    PRIMARY KEY (`id`),
    UNIQUE KEY `id` (`id`),
    INDEX idx_cloudserver_provider (`provider`),
    INDEX idx_cloudserver_instance_id (`instance_id`),
    INDEX idx_cloudserver_instance_name (`instance_name`),
    INDEX idx_cloudserver_zone (`zone`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci;