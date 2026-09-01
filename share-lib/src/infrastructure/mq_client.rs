//! MQ 原子操作（基于 lapin 的异步实现）
//!
//! 本模块只提供连接 / 建队 / 发布三个生产端原子操作；
//! 消费端的消费循环（consumer 建立、消息分发、ack 时机）与各 crate 业务强相关，
//! 继续留在各 crate 内实现。

use lapin::types::FieldTable;

use crate::data_structure::MailManErr;

/// 建立 MQ 连接
pub async fn connect(mq_str: &str) -> Result<lapin::Connection, MailManErr<'static, String>> {
    lapin::Connection::connect(mq_str, lapin::ConnectionProperties::default())
        .await
        .map_err(|e| {
            MailManErr::new(
                500,
                "Infrastructure: MQ connect",
                Some(format!("连接 MQ 失败: {e}")),
                1,
            )
        })
}

/// 声明 durable quorum 队列（幂等，已存在则直接通过）
pub async fn declare_quorum_queue(
    channel: &lapin::Channel,
    name: &str,
) -> Result<(), MailManErr<'static, String>> {
    let mut args = FieldTable::default();
    args.insert(
        "x-queue-type".into(),
        lapin::types::AMQPValue::LongString("quorum".into()),
    );

    channel
        .queue_declare(
            name.into(),
            lapin::options::QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            args,
        )
        .await
        .map_err(|e| {
            MailManErr::new(
                500,
                "Infrastructure: MQ declare queue",
                Some(format!("声明队列 {name} 失败: {e}")),
                1,
            )
        })?;

    Ok(())
}

/// 向默认交换机发布 JSON 消息（routing key 即队列名，持久化投递）
pub async fn publish_json(
    channel: &lapin::Channel,
    queue: &str,
    payload: &serde_json::Value,
) -> Result<(), MailManErr<'static, String>> {
    let body = serde_json::to_vec(payload).map_err(|e| {
        MailManErr::new(
            500,
            "Infrastructure: MQ publish",
            Some(format!("消息序列化失败: {e}")),
            1,
        )
    })?;

    channel
        .basic_publish(
            "".into(),
            queue.into(),
            lapin::options::BasicPublishOptions::default(),
            &body,
            lapin::BasicProperties::default()
                .with_content_type("application/json".into())
                .with_delivery_mode(2),
        )
        .await
        .map_err(|e| {
            MailManErr::new(
                500,
                "Infrastructure: MQ publish",
                Some(format!("向队列 {queue} 发布消息失败: {e}")),
                1,
            )
        })?;

    Ok(())
}
