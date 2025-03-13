-- Your SQL goes here
-- create job_log table
DROP TABLE IF EXISTS `job_log`;

CREATE TABLE `job_log` (
    `id` VARCHAR(255) NOT NULL COMMENT "job id",
    `type` VARCHAR(255) NOT NULL COMMENT "task type",
    `worker` VARCHAR(255) COMMENT "worker name",
    `status` TINYINT UNSIGNED NOT NULL COMMENT "status",
    `params` VARCHAR(255) NOT NULL DEFAULT '{}' COMMENT "task params",
    `result` VARCHAR(255) NOT NULL DEFAULT '{}' COMMENT "task result",
    `create_time` DATETIME NULL COMMENT "task create time",
    `finish_time` DATETIME NULL COMMENT "task finish time",
    `update_time` DATETIME NULL COMMENT "log update time",
    `comment` VARCHAR(255) NULL DEFAULT NULL COMMENT "task comment",
    PRIMARY KEY (`id`),
    KEY `type` (`type`),
    KEY `worker` (`worker`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci;