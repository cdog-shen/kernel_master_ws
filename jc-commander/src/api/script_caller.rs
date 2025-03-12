use std::sync::Arc;

use actix_web::{web, HttpResponse};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::Value;

use share_lib::data_structure::MailManOk;

use crate::config::server;
use crate::services::job_log;

// send sync task to message queue
pub async fn call_sync(
    req: web::Json<Value>,
    db_pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
    mq_pool: web::Data<Arc<lapin::Connection>>,
) -> HttpResponse {
    let channel = mq_pool.create_channel().await.unwrap();
    let queue = format!(
        "{}_sync",
        &server::GLOBAL_CONFIG.read().unwrap().mq_queue_prefix
    );
    let payload = serde_json::to_vec(&req.into_inner()).unwrap();

    let res = channel
        .basic_publish(
            "",
            &queue,
            lapin::options::BasicPublishOptions::default(),
            &payload,
            lapin::BasicProperties::default(),
        )
        .await;

    match res {
        Ok(res_data) => HttpResponse::Ok().json(MailManOk::new(
            200,
            "Sync task sending success",
            Some(format!("{:?}", res_data)),
        )),
        Err(err_data) => HttpResponse::InternalServerError().json(err_data.to_string()),
    }
}

// send sync task to message queue
pub async fn call_async(
    req: web::Json<Value>,
    db_pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
    mq_pool: web::Data<Arc<lapin::Connection>>,
) -> HttpResponse {
    let channel = mq_pool.create_channel().await.unwrap();
    let queue = format!(
        "{}_async",
        &server::GLOBAL_CONFIG.read().unwrap().mq_queue_prefix
    );
    let payload = serde_json::to_vec(&req.into_inner()).unwrap();

    let res = channel
        .basic_publish(
            "",
            &queue,
            lapin::options::BasicPublishOptions::default(),
            &payload,
            lapin::BasicProperties::default(),
        )
        .await;

    match res {
        Ok(res_data) => HttpResponse::Ok().json(MailManOk::new(
            200,
            "Sync task sending success",
            Some(format!("{:?}", res_data)),
        )),
        Err(err_data) => HttpResponse::InternalServerError().json(err_data.to_string()),
    }
}
