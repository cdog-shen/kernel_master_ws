//! MQ 消费端封装
//!
//! 连接建立（复用 `share_lib::infrastructure::mq_client`）、quorum 队列声明、
//! sync / async 双队列消费循环与 ack 逻辑统一收敛在本模块；
//! `main.rs` 只负责配置加载、logger 初始化与本模块调用。
//! 消费循环内毒消息由 `task::execute` 清洗后返回 Err，本模块照常 ack 丢弃。

use futures::StreamExt;
use lapin::{Channel, Consumer, options::*};
// log 宏展开不带 log:: 前缀，需自行引入
use log::info;

use share_lib::data_structure::{MailManErr, MailManOk};
use share_lib::infrastructure::mq_client;
use share_lib::log_info;

use crate::config::worker;
use crate::service::task;

/// 启动 MQ 消费循环（startup 阶段失败直接 panic，与原 main.rs 行为一致）
pub async fn run() {
    // connect to MQ
    log_info!("MQ Pool init");
    let mq_str = worker::GLOBAL_CONFIG.read().unwrap().mq_str.clone();
    let conn = match mq_client::connect(&mq_str).await {
        Ok(conn) => {
            MailManOk::new(200, "MQ connect success", None::<&str>);
            conn
        }
        Err(e) => {
            MailManErr::new(500, "MQ connection Failed", e.msg, 2);
            panic!("MQ connection Failed");
        }
    };

    // create channel
    log_info!("MQ Channel init");
    let channel: Channel = match conn.create_channel().await {
        Ok(channel) => {
            MailManOk::new(200, "MQ channel created", Some(channel.id()));
            channel
        }
        Err(e) => {
            MailManErr::new(500, "MQ channel - xxx create failed", Some(e), 2);
            panic!("MQ channel - xxx create failed")
        }
    };

    // define queue name
    let sync_queue = worker::GLOBAL_CONFIG
        .read()
        .unwrap()
        .mq_queue_prefix
        .clone()
        + "_sync";
    let async_queue = worker::GLOBAL_CONFIG
        .read()
        .unwrap()
        .mq_queue_prefix
        .clone()
        + "_async";

    // declare queues
    match mq_client::declare_quorum_queue(&channel, &sync_queue).await {
        Ok(_) => {
            MailManOk::new(200, "Sync queue declare success", None::<&str>);
        }
        Err(e) => {
            MailManErr::new(200, "Sync queue declare Failed", e.msg, 1);
            panic!("Sync queue declare Failed");
        }
    }

    match mq_client::declare_quorum_queue(&channel, &async_queue).await {
        Ok(_) => {
            MailManOk::new(200, "Async queue declare success", None::<&str>);
        }
        Err(e) => {
            MailManErr::new(200, "Async queue declare Failed", e.msg, 2);
            panic!("Async queue declare Failed");
        }
    }

    // create consumer for Sync channel
    log_info!("SYNC Consumer init");
    let sync_consumer = match channel
        .basic_consume(
            &sync_queue,
            "sync",
            BasicConsumeOptions::default(),
            lapin::types::FieldTable::default(),
        )
        .await
    {
        Ok(consumer) => {
            MailManOk::new(200, "Sync consumer declare success", None::<&str>);
            consumer
        }
        Err(e) => {
            MailManErr::new(200, "Sync consumer declare Failed", Some(e), 2);
            panic!("Sync consumer declare Failed");
        }
    };

    // create consumer for Async channel
    log_info!("ASYNC Consumer init");
    let async_consumer = match channel
        .basic_consume(
            &async_queue,
            "async",
            BasicConsumeOptions::default(),
            lapin::types::FieldTable::default(),
        )
        .await
    {
        Ok(consumer) => {
            MailManOk::new(200, "Async consumer declare success", None::<&str>);
            consumer
        }
        Err(e) => {
            MailManErr::new(200, "Async consumer declare Failed", Some(e), 2);
            panic!("Async consumer declare Failed");
        }
    };

    // async rt for different task
    log_info!("Consumer start");
    tokio::select! {
        _ = consume_loop(sync_consumer, "Recv new SYNC task") => {}
        _ = consume_loop(async_consumer, "Recv new ASYNC task") => {}
    };
}

/// 单队列消费循环：每条消息交 `task::execute` 编排，无论成败都 ack
/// （毒消息在清洗层已被拦截为 Err，不会进入 redelivery 循环）
async fn consume_loop(mut consumer: Consumer, recv_key: &'static str) {
    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            MailManOk::new(200, recv_key, None::<&str>);
            let _ = task::execute(&delivery.data).await;
            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("Failed to ack");
        }
    }
}
