-- 反向操作：删除新列，恢复旧列
ALTER TABLE notification_templates DROP COLUMN smtp;
ALTER TABLE notification_templates DROP COLUMN bark;
ALTER TABLE notification_templates DROP COLUMN gotify;
ALTER TABLE notification_templates DROP COLUMN ntfy;
ALTER TABLE notification_templates DROP COLUMN teams;
ALTER TABLE notification_templates DROP COLUMN webhook;

ALTER TABLE notification_templates ADD COLUMN channel_type VARCHAR(32) NOT NULL DEFAULT 'smtp';
ALTER TABLE notification_templates ADD COLUMN subject_template VARCHAR(512);
ALTER TABLE notification_templates ADD COLUMN content_template TEXT NOT NULL DEFAULT '';
ALTER TABLE notification_templates ADD COLUMN content_format VARCHAR(16);

-- 删除新格式预置模板
DELETE FROM notification_templates WHERE name IN (
    'urgent_alert', 'normal_notify', 'bark_silent', 'bark_timesensitive',
    'webhook_json', 'smtp_html_alert', 'all_channels_alert'
);

-- 恢复旧格式预置模板
INSERT INTO notification_templates (name, description, channel_type, subject_template, content_template, content_format, params_template, is_enabled)
VALUES
    ('bark_urgent_alert', 'Bark 紧急告警', 'bark', '[{{level}}] {{service}} 告警', '{{message}}', 'text', '{"level": "active", "sound": "alarm", "group": "{{group}}", "url": "{{url}}"}', true),
    ('bark_timesensitive_alert', 'Bark 时效性通知', 'bark', '[{{level}}] {{service}} 通知', '{{message}}', 'text', '{"level": "timeSensitive", "sound": "multiwayinvitation", "group": "{{group}}", "url": "{{url}}"}', true),
    ('bark_normal', 'Bark 普通通知', 'bark', '{{title}}', '{{message}}', 'text', '{"level": "active", "sound": "healthnotification", "group": "{{group}}", "url": "{{url}}"}', true),
    ('bark_silent', 'Bark 静默通知', 'bark', '{{title}}', '{{message}}', 'text', '{"level": "passive", "group": "{{group}}", "url": "{{url}}"}', true),
    ('gotify_urgent', 'Gotify 紧急告警', 'gotify', '[{{level}}] {{service}} 告警', '{{message}}', 'text', '{"priority": "8", "url": "{{url}}"}', true),
    ('gotify_normal', 'Gotify 普通通知', 'gotify', '{{title}}', '{{message}}', 'text', '{"priority": "5", "url": "{{url}}"}', true),
    ('teams_urgent', 'Teams 紧急告警', 'teams', '[{{level}}] {{service}} 告警', '{{message}}', 'text', '{"url": "{{url}}"}', true),
    ('teams_normal', 'Teams 普通通知', 'teams', '{{title}}', '{{message}}', 'text', '{"url": "{{url}}"}', true),
    ('teams_custom_json', 'Teams 自定义 JSON', 'teams', '{{title}}', '{{body}}', 'json', '{"id": "{{id}}", "title": "{{title}}", "body": "{{body}}"}', true),
    ('webhook_json', 'Webhook JSON', 'webhook', '{{title}}', '{{body}}', 'json', '{"title": "{{title}}", "body": "{{body}}", "level": "{{level}}"}', true),
    ('webhook_default', 'Webhook 默认', 'webhook', '{{title}}', '{{body}}', 'text', NULL, true);
