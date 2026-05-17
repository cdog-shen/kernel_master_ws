-- 为 notification_templates 表添加 params_template 列（Bark 等渠道的扩展参数模板）
ALTER TABLE notification_templates
    ADD COLUMN params_template JSONB;

-- 预置 Bark 模板：紧急告警（level=active，立即弹出，持续提醒）
INSERT INTO notification_templates (name, description, channel_type, subject_template, content_template, content_format, params_template, is_enabled)
VALUES (
    'bark_urgent_alert',
    'Bark 紧急告警 — 立即弹出，持续提醒',
    'bark',
    '[{{level}}] {{service}} 告警',
    '{{message}}',
    'text',
    '{"level": "active", "sound": "alarm", "group": "{{group}}", "url": "{{url}}"}',
    true
);

-- 预置 Bark 模板：时效性通知（level=timeSensitive，专注模式也可显示）
INSERT INTO notification_templates (name, description, channel_type, subject_template, content_template, content_format, params_template, is_enabled)
VALUES (
    'bark_timesensitive_alert',
    'Bark 时效性通知 — 专注模式下仍可显示',
    'bark',
    '[{{level}}] {{service}} 通知',
    '{{message}}',
    'text',
    '{"level": "timeSensitive", "sound": "multiwayinvitation", "group": "{{group}}", "url": "{{url}}"}',
    true
);

-- 预置 Bark 模板：普通通知（level=active，标准弹出）
INSERT INTO notification_templates (name, description, channel_type, subject_template, content_template, content_format, params_template, is_enabled)
VALUES (
    'bark_normal',
    'Bark 普通通知 — 标准弹出提醒',
    'bark',
    '{{title}}',
    '{{message}}',
    'text',
    '{"level": "active", "sound": "healthnotification", "group": "{{group}}", "url": "{{url}}"}',
    true
);

-- 预置 Bark 模板：静默通知（level=passive，不弹窗不响铃，仅通知中心）
INSERT INTO notification_templates (name, description, channel_type, subject_template, content_template, content_format, params_template, is_enabled)
VALUES (
    'bark_silent',
    'Bark 静默通知 — 不弹窗不响铃，仅出现在通知中心',
    'bark',
    '{{title}}',
    '{{message}}',
    'text',
    '{"level": "passive", "group": "{{group}}", "url": "{{url}}"}',
    true
);
