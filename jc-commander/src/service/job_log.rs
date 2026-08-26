use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::job_log::*;

// get job by id
pub async fn get_by_id<'a>(
    id: String,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, JobLogModel>, MailManErr<'a, String>> {
    match JobLogModel::get_log_by_id(&id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service: ID for Job", Some(msg))),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(400, "Service: ID for Job", Some(msg.1), 1)),
            _ => Err(MailManErr::new(500, "Service: ID for Job", Some(msg.1), 1)),
        },
    }
}

// get all job logs
pub async fn get_all<'a>(
    filter: Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Vec<JobLogModel>>, MailManErr<'a, String>> {
    match JobLogModel::get_logs_with_filter(&filter, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service: All Job", Some(msg))),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(400, "Service: All Job", Some(msg.1), 1)),
            _ => Err(MailManErr::new(500, "Service: All Job", Some(msg.1), 1)),
        },
    }
}

// create new job log
pub async fn new<'a>(
    data: JobLogInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match JobLogModel::new_log(&data, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: New Job",
            Some(Value::String(format!("Line changed: {msg}"))),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(400, "Service: New Job", Some(msg.1), 1)),
            _ => Err(MailManErr::new(500, "Service: New Job", Some(msg.1), 1)),
        },
    }
}

// update job log
//
// worker 回报任务结果：更新日志后向 DONE_TASK_LIST push 任务 id，
// 通知 service::script_caller::call_sync 的等待方
pub async fn update<'a>(
    data: JobLogInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match JobLogModel::update_log(&data, &mut pool.get().unwrap()) {
        Ok(msg) => {
            crate::service::script_caller::DONE_TASK_LIST.push(
                data.id
                    .clone()
                    .expect("TaskID missing")
                    .as_str()
                    .parse()
                    .expect("Not a valid UUID"),
            );
            Ok(MailManOk::new(
                200,
                "Service: Update Job",
                Some(Value::String(format!("Update log: {msg}"))),
            ))
        }
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(400, "Service: Update Job", Some(msg.1), 1)),
            _ => Err(MailManErr::new(500, "Service: Update Job", Some(msg.1), 1)),
        },
    }
}

// delete job log
pub async fn delete<'a>(
    id: String,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match JobLogModel::delete_log(&id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Delete Job",
            Some(Value::String(format!("Line changed: {msg}"))),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(400, "Service: Delete Job", Some(msg.1), 1)),
            _ => Err(MailManErr::new(500, "Service: Delete Job", Some(msg.1), 1)),
        },
    }
}
