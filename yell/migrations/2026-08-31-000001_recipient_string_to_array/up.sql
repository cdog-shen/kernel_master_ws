-- recipient 字段由纯 string 迁移为数组（存量 string 包装为单元素数组，保序）
-- 同时将 notification_records.recipient 放宽为 TEXT（数组 JSON 序列化后可能超过 512 字符）

UPDATE notification_aliases a
SET recipients = (
    SELECT jsonb_agg(
               jsonb_set(elem, '{recipient}', jsonb_build_array(elem->'recipient'))
               ORDER BY ord
           )
    FROM jsonb_array_elements(a.recipients) WITH ORDINALITY AS t(elem, ord)
);

ALTER TABLE notification_records ALTER COLUMN recipient TYPE TEXT;
