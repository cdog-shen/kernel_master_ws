-- 删除预置的 Bark 模板
DELETE FROM notification_templates WHERE name IN (
    'bark_urgent_alert',
    'bark_timesensitive_alert',
    'bark_normal',
    'bark_silent'
);

-- 移除 params_template 列
ALTER TABLE notification_templates
    DROP COLUMN IF EXISTS params_template;
