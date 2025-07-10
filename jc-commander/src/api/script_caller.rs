use std::sync::Arc;

use actix_web::{web, HttpResponse};
use chrono::{self, Local};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::Value;
use uuid::Uuid;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::config::server;
use crate::services::job_log;

// send sync task to message queue
pub async fn call_sync(
    req: web::Json<Value>,
    db_pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
    mq_pool: web::Data<Arc<lapin::Connection>>,
) -> HttpResponse {
    let channel = mq_pool.create_channel().await.unwrap();
    let queue_prefix = server::GLOBAL_CONFIG
        .read()
        .unwrap()
        .mq_queue_prefix
        .clone();
    let self_id = server::GLOBAL_CONFIG.read().unwrap().subsys_uuid.clone();
    let queue = format!("{}_sync", queue_prefix);

    let uuid = Uuid::new_v4().to_string();
    let mut req = req.into_inner();
    req["id"] = serde_json::Value::String(uuid.clone());
    req["commander"] = serde_json::Value::String(self_id.clone());
    let payload = serde_json::to_vec(&req).unwrap();
    let new_log_value = serde_json::json!(
            {"id": uuid.clone(),
            "script": req["script"],
            "exec_type": "sync",
            "commander": self_id,
            "status": 1,
            "params": req["params"].to_string(),
            "result": "{}",
            "create_time": Local::now().naive_local().format("%Y-%m-%dT%H:%M:%S").to_string(),
            "comment": req["comment"],
    });
    let new_log = new_log_value.as_object().unwrap();

    match job_log::new(new_log, &db_pool) {
        Ok(_) => (),
        Err(e) => return HttpResponse::InternalServerError().json(e),
    }

    match channel
        .basic_publish(
            "",
            &queue,
            lapin::options::BasicPublishOptions::default(),
            &payload,
            lapin::BasicProperties::default(),
        )
        .await
    {
        Ok(res_data) => {
            MailManOk::new(
                200,
                "Sync task send success",
                Some(format!("{:?}", res_data)),
            );
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(MailManErr::new(
                500,
                "Task sending Failed",
                e,
                1,
            ))
        }
    }

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        match job_log::get_by_id(uuid.clone(), &db_pool) {
            Ok(MailManOk {
                code: _,
                key: _,
                data: job_log_res,
            }) => match job_log_res {
                Some(job_log) => {
                    if job_log.status == 0 {
                        return HttpResponse::InternalServerError().json(MailManErr::new(
                            500,
                            "Job execute Error",
                            format!("{:?}", job_log),
                            1,
                        ));
                    } else if job_log.status == 2 {
                        return HttpResponse::Ok().json(MailManOk::new(
                            200,
                            "Sync task called success",
                            Some(job_log),
                        ));
                    }
                }
                None => {
                    return HttpResponse::InternalServerError().json(MailManErr::new(
                        500,
                        "no job found",
                        uuid,
                        1,
                    ))
                }
            },
            Err(e) => return HttpResponse::InternalServerError().json(e),
        }
    }
}

// send async task to message queue
pub async fn call_async(
    req: web::Json<Value>,
    db_pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
    mq_pool: web::Data<Arc<lapin::Connection>>,
) -> HttpResponse {
    let channel = mq_pool.create_channel().await.unwrap();
    let queue_prefix = server::GLOBAL_CONFIG
        .read()
        .unwrap()
        .mq_queue_prefix
        .clone();
    let self_id = server::GLOBAL_CONFIG.read().unwrap().subsys_uuid.clone();
    let queue = format!("{}_async", queue_prefix);

    let uuid = Uuid::new_v4().to_string();
    let mut req = req.into_inner();
    req["id"] = serde_json::Value::String(uuid.clone());
    req["commander"] = serde_json::Value::String(self_id.clone());
    let payload = serde_json::to_vec(&req).unwrap();
    let new_log_value = serde_json::json!(
            {"id": uuid.clone(),
            "script": req["script"],
            "exec_type": "async",
            "commander": self_id,
            "status": 1,
            "params": req["params"].to_string(),
            "result": "{}",
            "create_time": Local::now().naive_local().format("%Y-%m-%dT%H:%M:%S").to_string(),
            "comment": req["comment"],
    });
    let new_log = new_log_value.as_object().unwrap();

    match job_log::new(new_log, &db_pool) {
        Ok(_) => (),
        Err(e) => return HttpResponse::InternalServerError().json(e),
    }

    match channel
        .basic_publish(
            "",
            &queue,
            lapin::options::BasicPublishOptions::default(),
            &payload,
            lapin::BasicProperties::default(),
        )
        .await
    {
        Ok(_) => {
            return HttpResponse::Ok().json(MailManOk::new(
                200,
                "Async task send success",
                Some(uuid),
            ))
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(MailManErr::new(
                500,
                "Task sending Failed",
                e,
                1,
            ))
        }
    }
}
