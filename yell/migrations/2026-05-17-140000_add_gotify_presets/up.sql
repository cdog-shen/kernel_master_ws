-- 预置 Gotify 模板：紧急告警（priority=8，高优先级推送）
INSERT INTO notification_templates (name, description, channel_type, subject_template, content_template, content_format, params_template, is_enabled)
VALUES (
    'gotify_urgent',
    'Gotify 紧急告警 — priority=8，高优先级推送',
    'gotify',
    '[{{level}}] {{service}} 告警',
    '{{message}}',
    'text',
    '{"priority": "8", "url": "{{url}}"}',
    true
);

-- 预置 Gotify 模板：普通通知（priority=5，标准优先级）
INSERT INTO notification_templates (name, description, channel_type, subject_template, content_template, content_format, params_template, is_enabled)
VALUES (
    'gotify_normal',
    'Gotify 普通通知 — priority=5，标准优先级',
    'gotify',
    '{{title}}',
    '{{message}}',
    'text',
    '{"priority": "5", "url": "{{url}}"}',
    true
);
