-- Your SQL goes here
-- create job_log table
DROP TABLE IF EXISTS `job_log`;

CREATE TABLE `job_log` (
    `id` VARCHAR(255) NOT NULL COMMENT "job id",
    `script` VARCHAR(255) NOT NULL COMMENT "script name",
    `exec_type` VARCHAR(255) NOT NULL COMMENT "task type",
    `commander` VARCHAR(255) NOT NULL COMMENT "commander id",
    `worker` VARCHAR(255) COMMENT "worker name",
    `status` TINYINT UNSIGNED NOT NULL COMMENT "status",
    `params` TEXT NOT NULL COMMENT "task params",
    `result` TEXT NOT NULL COMMENT "task result",
    `create_time` DATETIME NULL COMMENT "task create time",
    `finish_time` DATETIME NULL COMMENT "task finish time",
    `update_time` DATETIME NULL COMMENT "log update time",
    `comment` VARCHAR(255) NULL DEFAULT NULL COMMENT "task comment",
    PRIMARY KEY (`id`),
    KEY `exec_type` (`exec_type`),
    KEY `worker` (`worker`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci;

-- create cron_job table
DROP TABLE IF EXISTS `cron_job`;

CREATE TABLE `cron_job` (
    `id` VARCHAR(255) NOT NULL COMMENT "job id",
    `script` VARCHAR(255) NOT NULL COMMENT "script name",
    `frequency` BIGINT NOT NULL DEFAULT 120 COMMENT "frequency",
    `launch_at` DATETIME NOT NULL COMMENT "first run time",
    `times` INT UNSIGNED NOT NULL DEFAULT 0 COMMENT "run times",
    `status` TINYINT UNSIGNED NOT NULL DEFAULT 0 COMMENT "status",
    `params` TEXT NOT NULL COMMENT "task params",
    `create_time` DATETIME NOT NULL COMMENT "task create time",
    `update_time` DATETIME NOT NULL COMMENT "log update time",
    `comment` VARCHAR(255) NOT NULL DEFAULT "" COMMENT "task comment",
    PRIMARY KEY (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci;