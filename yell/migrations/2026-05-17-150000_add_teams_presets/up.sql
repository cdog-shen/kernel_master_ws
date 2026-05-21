-- 预置 Teams 模板：紧急告警
INSERT INTO notification_templates (name, description, channel_type, subject_template, content_template, content_format, params_template, is_enabled)
VALUES (
    'teams_urgent',
    'Teams 紧急告警 — 红色主题卡片',
    'teams',
    '[{{level}}] {{service}} 告警',
    '{{message}}',
    'text',
    '{"url": "{{url}}"}',
    true
);

-- 预置 Teams 模板：普通通知
INSERT INTO notification_templates (name, description, channel_type, subject_template, content_template, content_format, params_template, is_enabled)
VALUES (
    'teams_normal',
    'Teams 普通通知 — 蓝色主题卡片',
    'teams',
    '{{title}}',
    '{{message}}',
    'text',
    '{"url": "{{url}}"}',
    true
);
