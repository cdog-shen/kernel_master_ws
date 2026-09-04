-- recipient 字段由纯 string 迁移为数组（存量 string 包装为单元素数组，保序）
-- 同时将 notification_records.recipient 放宽为 TEXT（数组 JSON 序列化后可能超过 512 字符）

-- 仅处理 recipients 为数组的行（存量标量行原样保留，需人工检查后另行处理）；
-- 元素内 recipient 已是数组的保持不变（幂等，可重复执行）
UPDATE notification_aliases a
SET recipients = (
    SELECT jsonb_agg(
               CASE
                   WHEN jsonb_typeof(elem) = 'object'
                        AND elem ? 'recipient'
                        AND jsonb_typeof(elem->'recipient') IS DISTINCT FROM 'array'
                   THEN jsonb_set(elem, '{recipient}', jsonb_build_array(elem->'recipient'))
                   ELSE elem
               END
               ORDER BY ord
           )
    FROM jsonb_array_elements(a.recipients) WITH ORDINALITY AS t(elem, ord)
)
WHERE jsonb_typeof(a.recipients) = 'array';

ALTER TABLE notification_records ALTER COLUMN recipient TYPE TEXT;
