-- 预置 Teams 自定义 JSON 模板（content_format=json，params_template 整体作为 payload）
INSERT INTO notification_templates (name, description, channel_type, subject_template, content_template, content_format, params_template, is_enabled)
VALUES (
    'teams_custom_json',
    'Teams 自定义 JSON — params_template 整体作为 webhook payload',
    'teams',
    '{{title}}',
    '{{body}}',
    'json',
    '{"id": "{{id}}", "title": "{{title}}", "body": "{{body}}"}',
    true
);
