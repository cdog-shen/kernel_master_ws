-- ============================================================
-- 模板渠道变更：移除 ntfy 渠道与 params_template 字段，
-- teams 列改名 teams_hook（RENAME 自然保留存量数据）
-- ============================================================

ALTER TABLE notification_templates DROP COLUMN ntfy;
ALTER TABLE notification_templates RENAME COLUMN teams TO teams_hook;
ALTER TABLE notification_templates DROP COLUMN params_template;
