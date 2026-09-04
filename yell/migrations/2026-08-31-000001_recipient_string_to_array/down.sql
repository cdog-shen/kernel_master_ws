-- 回滚：单元素数组还原为 string；多元素数组取第一个元素（有损，仅用于回滚兜底）
-- notification_records.recipient 改回 VARCHAR(512)（若已有超长数据此步会失败，需手工处理）

-- 仅处理 recipients 为数组的行（与 up 保持一致的容错口径）
UPDATE notification_aliases a
SET recipients = (
    SELECT jsonb_agg(
               CASE
                   WHEN jsonb_typeof(elem) = 'object' AND elem ? 'recipient'
                   THEN jsonb_set(elem, '{recipient}',
                           CASE WHEN jsonb_typeof(elem->'recipient') = 'array'
                                THEN elem->'recipient'->0
                                ELSE elem->'recipient'
                           END)
                   ELSE elem
               END
               ORDER BY ord
           )
    FROM jsonb_array_elements(a.recipients) WITH ORDINALITY AS t(elem, ord)
)
WHERE jsonb_typeof(a.recipients) = 'array';

ALTER TABLE notification_records ALTER COLUMN recipient TYPE VARCHAR(512);
