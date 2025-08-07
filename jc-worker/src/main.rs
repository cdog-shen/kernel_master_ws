// std import
use log::info;
// rt import
use futures::StreamExt;
// MQ import
use lapin::{Channel, Connection, ConnectionProperties, options::*, types::FieldTable};
// share-lib import
use share_lib::data_structure::{MailManErr, MailManOk};
use share_lib::{log_info, logger};
// local import
use config::worker;
use service::task;
// local modules
mod config;
mod service;

#[tokio::main]
async fn main() {
    // reload config
    match worker::GLOBAL_CONFIG.write().unwrap().reload() {
        Ok(_) => {
            MailManOk::new(200, "config load DONE", None::<&str>);
        }
        Err(e) => {
            MailManErr::new(500, "config load Failed", Some(e), 2);
            panic!("config load Failed");
        }
    }

    // config out put
    println!("config is {:#?}", &*worker::GLOBAL_CONFIG);

    // init logger
    logger::init_logger(
        &worker::GLOBAL_CONFIG.read().unwrap().log_path,
        &worker::GLOBAL_CONFIG.read().unwrap().log_level,
    );

    // connect to MQ
    log_info!("MQ Pool init");
    let mq_str = worker::GLOBAL_CONFIG.read().unwrap().mq_str.clone();
    let conn = match Connection::connect(&mq_str, ConnectionProperties::default()).await {
        Ok(conn) => {
            MailManOk::new(200, "MQ connect success", None::<&str>);
            conn
        }
        Err(e) => {
            MailManErr::new(500, "MQ connection Failed", Some(e), 2);
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
    match channel
        .queue_declare(
            &sync_queue,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            {
                let mut args = FieldTable::default();
                args.insert(
                    "x-queue-type".into(),
                    lapin::types::AMQPValue::LongString("quorum".into()),
                );
                args
            },
        )
        .await
    {
        Ok(_) => {
            MailManOk::new(200, "Sync queue declare success", None::<&str>);
        }
        Err(e) => {
            MailManErr::new(200, "Sync queue declare Failed", Some(e), 1);
            panic!("Sync queue declare Failed");
        }
    }

    match channel
        .queue_declare(
            &async_queue,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            {
                let mut args = FieldTable::default();
                args.insert(
                    "x-queue-type".into(),
                    lapin::types::AMQPValue::LongString("quorum".into()),
                );
                args
            },
        )
        .await
    {
        Ok(_) => {
            MailManOk::new(200, "Async queue declare success", None::<&str>);
        }
        Err(e) => {
            MailManErr::new(200, "Async queue declare Failed", Some(e), 2);
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
            FieldTable::default(),
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
            FieldTable::default(),
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
        _ = async {
            let mut consumer = sync_consumer;
            while let Some(delivery) = consumer.next().await {
                if let Ok(delivery) = delivery {
                    MailManOk::new(200, "Recv new SYNC task", None::<&str>);
                    let _ = task::execute(&delivery.data).await;
                    delivery.ack(BasicAckOptions::default()).await.expect("Failed to ack");
                }
            }
        } => {}
        _ = async {
            let mut consumer = async_consumer;
            while let Some(delivery) = consumer.next().await {
                if let Ok(delivery) = delivery {
                    MailManOk::new(200, "Recv new ASYNC task", None::<&str>);
                    let _ = task::execute(&delivery.data).await;
                    delivery.ack(BasicAckOptions::default()).await.expect("Failed to ack");
                }
            }
        } => {}
    };
}
