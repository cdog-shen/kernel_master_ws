//! 脚本调用编排层
//!
//! call_sync / call_async 的完整编排：
//! 拼 payload → 写 job_log → MQ 投递 →（sync）轮询 DONE_TASK_LIST 等待
//! worker 回报 → 查结果 → 按 status 组装响应。
//!
//! 响应组装留在本层（沿用 handler 时期的对外结构，含 Ok(500) 这类历史形态），
//! handler 仅做请求清洗与本层结果的透传。

use std::sync::Arc;

use actix_web::{HttpResponse, web};
use chrono::{self, Local};
use crossbeam::queue::SegQueue;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use once_cell::sync::Lazy;
use serde_json::{Value, json};
use uuid::Uuid;

use share_lib::data_structure::{MailManErr, MailManOk};
use share_lib::err_mapping::MailManErrResponser;
use share_lib::infrastructure::mq_client;

use crate::config::server;
use crate::model::job_log::JobLogInfo;
use crate::service::job_log;
use crate::util::{mq_async_queue, mq_sync_queue};

/// sync 任务完成通知全局队列
///
/// worker 回报（`job_log::update`）时 push 任务 id；
/// `call_sync` 轮询等待自身 id 出现以判定 sync 任务执行完毕。
/// 随编排归位于此（原为 main 中创建、经 app_data 注入的 SegQueue）。
pub static DONE_TASK_LIST: Lazy<SegQueue<Uuid>> = Lazy::new(SegQueue::new);

// send sync task to message queue
pub async fn call_sync(
    req: Value,
    db_pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    mq_pool: &web::Data<Arc<lapin::Connection>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let channel = mq_pool.create_channel().await.unwrap();
    let self_id = server::GLOBAL_CONFIG.read().unwrap().subsys_uuid.clone();
    let queue = mq_sync_queue();

    let uuid = Uuid::new_v4();
    let mut req = req;
    let timeout = req["timeout"].as_u64().unwrap_or(30);
    req["id"] = serde_json::Value::String(uuid.to_string().clone());
    req["commander"] = serde_json::Value::String(self_id.clone());
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

    match job_log::new(new_log, db_pool).await {
        Ok(_) => (),
        Err(e) => return Err(MailManErrResponser::mapping_from_mme(e)),
    }

    mq_client::publish_json(&channel, &queue, &req)
        .await
        .map_err(|e| {
            MailManErrResponser::mapping_from_mme(MailManErr::new(
                500,
                "Task sending Failed",
                e.msg,
                1,
            ))
        })?;
    MailManOk::new(200, "Sync task send success", None::<&str>);

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
        while let Some(id) = DONE_TASK_LIST.pop() {
            if id == uuid {
                found = true;
                break;
            } else {
                temp_vec.push(id);
            }
        }
        // Push back all non-matching UUIDs
        for id in temp_vec {
            DONE_TASK_LIST.push(id);
        }
        if found {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }

    match job_log::get_by_id(uuid.to_string().clone(), db_pool).await {
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
    req: Value,
    db_pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    mq_pool: &web::Data<Arc<lapin::Connection>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let channel = mq_pool.create_channel().await.unwrap();
    let self_id = server::GLOBAL_CONFIG.read().unwrap().subsys_uuid.clone();
    let queue = mq_async_queue();

    let uuid = Uuid::new_v4().to_string();
    let mut req = req;
    req["id"] = serde_json::Value::String(uuid.clone());
    req["commander"] = serde_json::Value::String(self_id.clone());
    let new_log_value = serde_json::json!({
            "id": uuid.to_string().clone(),
            "script": req["script"],
            "exec_type": "async",
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

    match job_log::new(new_log, db_pool).await {
        Ok(_) => (),
        Err(e) => return Err(MailManErrResponser::mapping_from_mme(e)),
    }

    match mq_client::publish_json(&channel, &queue, &req).await {
        Ok(_) => {
            Ok(HttpResponse::Ok().json(MailManOk::new(200, "Async task send success", Some(uuid))))
        }
        Err(e) => Err(MailManErrResponser::mapping_from_mme(MailManErr::new(
            500,
            "Task sending Failed",
            e.msg,
            1,
        ))),
    }
}
