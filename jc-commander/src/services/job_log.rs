use actix_web::web;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::{Map, Value};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::models::job_log::*;

// get all job logs
pub fn get_all<'a>(
    filter: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Vec<Value>>, MailManErr<'a>> {
    match JobLogModel::get_logs_with_filter(filter.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "All job logs", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

// create new job log
pub fn new<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a>> {
    match JobLogModel::new_log(data.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "New job log",
            Some(Value::String(format!("New log: {}", msg))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

// update job log
pub fn update<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a>> {
    match JobLogModel::update_log(data.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Update job log",
            Some(Value::String(format!("Update log: {}", msg))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

// delete job log
pub fn delete<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a>> {
    match JobLogModel::delete_log(
        match data.get("id").and_then(Value::as_str) {
            Some(value) => value.to_string(),
            None => return Err(MailManErr::new(400, "Bad requests", "id not found", 1)),
        },
        &mut pool.get().unwrap(),
    ) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Delete job log",
            Some(Value::String(format!("Delete log: {}", msg))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}
