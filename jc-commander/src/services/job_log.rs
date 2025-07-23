use actix_web::web;
use crossbeam::queue::SegQueue;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::{Map, Value};
use uuid::Uuid;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::models::job_log::*;

// get job by id
pub fn get_by_id<'a>(
    id: String,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, JobLogModel>, MailManErr<'a, String>> {
    match JobLogModel::get_log_by_id(id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Job log found", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    }
}

// get all job logs
pub fn get_all<'a>(
    filter: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Vec<Value>>, MailManErr<'a, String>> {
    match JobLogModel::get_logs_with_filter(filter.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "All job logs", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    }
}

// create new job log
pub fn new<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match JobLogModel::new_log(data.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "New job log",
            Some(Value::String(format!("New log: {msg}"))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    }
}

// update job log
pub fn update<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
    done_task_list: &web::Data<SegQueue<Uuid>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match JobLogModel::update_log(data.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => {
            done_task_list.push(
                data.clone()
                    .get("id")
                    .expect("TaskID missing")
                    .as_str()
                    .unwrap()
                    .parse()
                    .expect("Not a valid UUID"),
            );
            Ok(MailManOk::new(
                200,
                "Update job log",
                Some(Value::String(format!("Update log: {msg}"))),
            ))
        }
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    }
}

// delete job log
pub fn delete<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match JobLogModel::delete_log(
        match data.get("id").and_then(Value::as_str) {
            Some(value) => value.to_string(),
            None => {
                return Err(MailManErr::new(
                    400,
                    "Bad requests",
                    Some("id not found".to_string()),
                    1,
                ))
            }
        },
        &mut pool.get().unwrap(),
    ) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Delete job log",
            Some(Value::String(format!("Delete log: {msg}"))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    }
}
