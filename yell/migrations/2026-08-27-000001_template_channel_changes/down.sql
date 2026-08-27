-- 反向操作：加回 ntfy/params_template 列，teams_hook 改回 teams
ALTER TABLE notification_templates ADD COLUMN ntfy JSONB;
ALTER TABLE notification_templates RENAME COLUMN teams_hook TO teams;
ALTER TABLE notification_templates ADD COLUMN params_template JSONB;
