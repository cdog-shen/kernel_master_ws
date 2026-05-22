-- 预置 Webhook 模板：通用 JSON 通知
INSERT INTO notification_templates (name, description, channel_type, subject_template, content_template, content_format, params_template, is_enabled)
VALUES (
    'webhook_json',
    '通用 Webhook — content_format=json，params_template 整体作为 payload',
    'webhook',
    '{{title}}',
    '{{body}}',
    'json',
    '{"title": "{{title}}", "body": "{{body}}", "level": "{{level}}"}',
    true
);

-- 预置 Webhook 模板：默认格式
INSERT INTO notification_templates (name, description, channel_type, subject_template, content_template, content_format, params_template, is_enabled)
VALUES (
    'webhook_default',
    '通用 Webhook — 默认 JSON 格式（title/body/priority）',
    'webhook',
    '{{title}}',
    '{{body}}',
    'text',
    NULL,
    true
);
