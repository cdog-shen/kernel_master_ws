-- ============================================================
-- 模板机制重构：移除单渠道绑定，改为多渠道 JSONB 字段
-- ============================================================

-- 移除旧列
ALTER TABLE notification_templates DROP COLUMN channel_type;
ALTER TABLE notification_templates DROP COLUMN subject_template;
ALTER TABLE notification_templates DROP COLUMN content_template;
ALTER TABLE notification_templates DROP COLUMN content_format;

-- 添加各渠道 JSONB 列
ALTER TABLE notification_templates ADD COLUMN smtp JSONB;
ALTER TABLE notification_templates ADD COLUMN bark JSONB;
ALTER TABLE notification_templates ADD COLUMN gotify JSONB;
ALTER TABLE notification_templates ADD COLUMN ntfy JSONB;
ALTER TABLE notification_templates ADD COLUMN teams JSONB;
ALTER TABLE notification_templates ADD COLUMN webhook JSONB;

-- ============================================================
-- 删除旧格式预置模板
-- ============================================================
DELETE FROM notification_templates WHERE name IN (
    'bark_urgent_alert', 'bark_timesensitive_alert', 'bark_normal', 'bark_silent',
    'gotify_urgent', 'gotify_normal',
    'teams_urgent', 'teams_normal', 'teams_custom_json',
    'webhook_json', 'webhook_default'
);

-- ============================================================
-- 插入新格式预置模板（多渠道统一模板）
-- ============================================================

-- 紧急告警模板（Bark + Gotify + Teams）
INSERT INTO notification_templates (name, description, bark, gotify, teams, params_template, is_enabled)
VALUES (
    'urgent_alert',
    '紧急告警 — Bark/Gotify/Teams 三渠道联动',
    '{"title": "[{{level}}] {{service}} 告警", "body": "{{message}}", "level": "timeSensitive", "sound": "alarm", "group": "{{group}}", "url": "{{url}}"}',
    '{"title": "[{{level}}] {{service}} 告警", "message": "{{message}}", "priority": 8, "url": "{{url}}"}',
    '{"title": "[{{level}}] {{service}} 告警", "text": "{{message}}", "url": "{{url}}"}',
    NULL,
    true
);

-- 普通通知模板（Bark + Gotify + Teams）
INSERT INTO notification_templates (name, description, bark, gotify, teams, params_template, is_enabled)
VALUES (
    'normal_notify',
    '普通通知 — Bark/Gotify/Teams 三渠道联动',
    '{"title": "{{title}}", "body": "{{message}}", "level": "active", "sound": "healthnotification", "group": "{{group}}", "url": "{{url}}"}',
    '{"title": "{{title}}", "message": "{{message}}", "priority": 5, "url": "{{url}}"}',
    '{"title": "{{title}}", "text": "{{message}}", "url": "{{url}}"}',
    NULL,
    true
);

-- Bark 静默通知
INSERT INTO notification_templates (name, description, bark, params_template, is_enabled)
VALUES (
    'bark_silent',
    'Bark 静默通知 — 不弹窗不响铃，仅通知中心',
    '{"title": "{{title}}", "body": "{{message}}", "level": "passive", "group": "{{group}}"}',
    NULL,
    true
);

-- Bark 时效性通知（专注模式也可显示）
INSERT INTO notification_templates (name, description, bark, params_template, is_enabled)
VALUES (
    'bark_timesensitive',
    'Bark 时效性通知 — 专注模式下仍可显示',
    '{"title": "[{{level}}] {{service}} 通知", "body": "{{message}}", "level": "timeSensitive", "sound": "multiwayinvitation", "group": "{{group}}", "url": "{{url}}"}',
    NULL,
    true
);

-- Webhook JSON 模板
INSERT INTO notification_templates (name, description, webhook, params_template, is_enabled)
VALUES (
    'webhook_json',
    'Webhook JSON — 自定义 JSON payload',
    '{"title": "{{title}}", "body": "{{body}}", "level": "{{level}}"}',
    NULL,
    true
);

-- SMTP 邮件模板
INSERT INTO notification_templates (name, description, smtp, params_template, is_enabled)
VALUES (
    'smtp_html_alert',
    'SMTP 邮件 — HTML 格式告警',
    '{"subject": "[{{level}}] {{service}} 告警", "content_type": "html", "body": "<h2>{{level}} — {{service}}</h2><p>{{message}}</p><p><a href=\"{{url}}\">查看详情</a></p>"}',
    NULL,
    true
);

-- 全渠道模板示例（同时配置 SMTP + Bark + Gotify + Teams + Webhook）
INSERT INTO notification_templates (name, description, smtp, bark, gotify, teams, webhook, params_template, is_enabled)
VALUES (
    'all_channels_alert',
    '全渠道告警模板 — 同时配置所有渠道',
    '{"subject": "[{{level}}] {{service}} 告警", "content_type": "html", "body": "<h2>{{level}} — {{service}}</h2><p>{{message}}</p>"}',
    '{"title": "[{{level}}] {{service}} 告警", "body": "{{message}}", "level": "timeSensitive", "sound": "alarm", "group": "{{group}}", "url": "{{url}}"}',
    '{"title": "[{{level}}] {{service}} 告警", "message": "{{message}}", "priority": 8, "url": "{{url}}"}',
    '{"title": "[{{level}}] {{service}} 告警", "text": "{{message}}", "url": "{{url}}"}',
    '{"title": "{{level}} — {{service}}", "body": "{{message}}", "url": "{{url}}"}',
    NULL,
    true
);
