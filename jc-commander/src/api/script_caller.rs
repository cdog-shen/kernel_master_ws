use crossbeam::queue::SegQueue;
use std::sync::Arc;

use actix_web::{HttpResponse, web};
use chrono::{self, Local};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Value, json};
use uuid::Uuid;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::{config::server, model::job_log::JobLogInfo};
use crate::{service::job_log, util::err_mapping::MailManErrResponser};

// send sync task to message queue
pub async fn call_sync(
    req: web::Json<Value>,
    db_pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
    mq_pool: web::Data<Arc<lapin::Connection>>,
    done_task_list: web::Data<SegQueue<Uuid>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let channel = mq_pool.create_channel().await.unwrap();
    let queue_prefix = server::GLOBAL_CONFIG
        .read()
        .unwrap()
        .mq_queue_prefix
        .clone();
    let self_id = server::GLOBAL_CONFIG.read().unwrap().subsys_uuid.clone();
    let queue = format!("{queue_prefix}_sync");

    let uuid = Uuid::new_v4();
    let mut req = req.into_inner();
    let timeout = req["timeout"].as_u64().unwrap_or(30);
    req["id"] = serde_json::Value::String(uuid.to_string().clone());
    req["commander"] = serde_json::Value::String(self_id.clone());
    let payload = serde_json::to_vec(&req).unwrap();
    let new_log_value = serde_json::json!({
            "id": uuid.to_string().clone(),
            "script": req["script"],
            "exec_type": "sync",
            "commander": self_id,
            "worker": "None",
            "status": 1,
            "params": req["params"].to_string(),
            "result": "{}",
            "update_time": Local::now().naive_local().format("%Y-%m-%dT%H:%M:%S").to_string(),
            "finish_time": "None",
            "comment": req["comment"],
    });
    let new_log = JobLogInfo::from_map(serde_json::from_value(new_log_value).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(
            500,
            "Server Error",
            Some(e.to_string()),
            1,
        ))
    })?)
    .map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(500, "Server Error", Some(e), 1))
    })?;

    match job_log::new(new_log, &db_pool) {
        Ok(_) => (),
        Err(e) => return Err(MailManErrResponser::mapping_from_mme(e)),
    }

    match channel
        .basic_publish(
            "",
            &queue,
            lapin::options::BasicPublishOptions::default(),
            &payload,
            lapin::BasicProperties::default()
                .with_content_type("application/json".into())
                .with_delivery_mode(2),
        )
        .await
    {
        Ok(res_data) => {
            MailManOk::new(200, "Sync task send success", Some(format!("{res_data:?}")));
        }
        Err(e) => {
            return Err(MailManErrResponser::mapping_from_mme(MailManErr::new(
                500,
                "Task sending Failed",
                Some(e.to_string()),
                1,
            )));
        }
    }

    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > std::time::Duration::from_secs(timeout) {
            return Err(MailManErrResponser::mapping_from_mme(MailManErr::new(
                504,
                "Job execute Timeout",
                Some(uuid.to_string()),
                1,
            )));
        }

        // Instead of popping, check if the UUID exists in the queue
        let mut found = false;
        let mut temp_vec = Vec::new();
        while let Some(id) = done_task_list.pop() {
            if id == uuid {
                found = true;
                break;
            } else {
                temp_vec.push(id);
            }
        }
        // Push back all non-matching UUIDs
        for id in temp_vec {
            done_task_list.push(id);
        }
        if found {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }

    match job_log::get_by_id(uuid.to_string().clone(), &db_pool) {
        Ok(MailManOk {
            code: _,
            key: _,
            data: job_log_res,
        }) => match job_log_res {
            Some(job_log) => {
                if job_log.status == 2 {
                    Ok(HttpResponse::Ok().json(MailManOk::new(
                        200,
                        "Sync task called success",
                        Some(json!(job_log)),
                    )))
                } else {
                    MailManErr::new(500, "Job execute Error", Some(json!(job_log)), 1);
                    Ok(HttpResponse::InternalServerError().json(MailManErr::new(
                        500,
                        "Job execute Error",
                        Some(json!(job_log)),
                        1,
                    )))
                }
            }
            None => Err(MailManErrResponser::mapping_from_mme(MailManErr::new(
                500,
                "no job found",
                Some(uuid.to_string()),
                1,
            ))),
        },
        Err(e) => Ok(HttpResponse::InternalServerError().json(e)),
    }
}

// send async task to message queue
pub async fn call_async(
    req: web::Json<Value>,
    db_pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
    mq_pool: web::Data<Arc<lapin::Connection>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let channel = mq_pool.create_channel().await.unwrap();
    let queue_prefix = server::GLOBAL_CONFIG
        .read()
        .unwrap()
        .mq_queue_prefix
        .clone();
    let self_id = server::GLOBAL_CONFIG.read().unwrap().subsys_uuid.clone();
    let queue = format!("{queue_prefix}_async");

    let uuid = Uuid::new_v4().to_string();
    let mut req = req.into_inner();
    req["id"] = serde_json::Value::String(uuid.clone());
    req["commander"] = serde_json::Value::String(self_id.clone());
    let payload = serde_json::to_vec(&req).unwrap();
    let new_log_value = serde_json::json!({
            "id": uuid.to_string().clone(),
            "script": req["script"],
            "exec_type": "sync",
            "commander": self_id,
            "worker": "None",
            "status": 1,
            "params": req["params"].to_string(),
            "result": "{}",
            "update_time": Local::now().naive_local().format("%Y-%m-%dT%H:%M:%S").to_string(),
            "finish_time": "None",
            "comment": req["comment"],
    });
    let new_log = JobLogInfo::from_map(serde_json::from_value(new_log_value).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(
            500,
            "Server Error",
            Some(e.to_string()),
            1,
        ))
    })?)
    .map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(500, "Server Error", Some(e), 1))
    })?;

    match job_log::new(new_log, &db_pool) {
        Ok(_) => (),
        Err(e) => return Err(MailManErrResponser::mapping_from_mme(e)),
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
            Ok(HttpResponse::Ok().json(MailManOk::new(200, "Async task send success", Some(uuid))))
        }
        Err(e) => Err(MailManErrResponser::mapping_from_mme(MailManErr::new(
            500,
            "Task sending Failed",
            Some(e.to_string()),
            1,
        ))),
    }
}
