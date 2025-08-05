use actix_web::web;
use crossbeam::queue::SegQueue;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use uuid::Uuid;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::job_log::*;

// get job by id
pub fn get_by_id<'a>(
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
pub fn get_all<'a>(
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
pub fn new<'a>(
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
pub fn update<'a>(
    data: JobLogInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
    done_task_list: &web::Data<SegQueue<Uuid>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match JobLogModel::update_log(&data, &mut pool.get().unwrap()) {
        Ok(msg) => {
            done_task_list.push(
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
pub fn delete<'a>(
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
