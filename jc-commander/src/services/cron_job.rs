use std::sync::Mutex;

use actix_web::web;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::{Map, Value};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::models::cron_job::*;

// get cron by filter
pub fn get_all<'a>(
    filter: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Vec<Value>>, MailManErr<'a>> {
    match CronJobModel::get_crons_with_filter(filter.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "All cron jobs", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

// new cron job
pub fn new<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a>> {
    match CronJobModel::new_cron(data.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "New cron job",
            Some(Value::String(format!("New cron: {}", msg))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

// update cron job
pub fn update<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a>> {
    match CronJobModel::update_cron(data.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Update cron job",
            Some(Value::String(format!("Update cron: {}", msg))),
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
    match CronJobModel::delete_cron(
        match data.get("id").and_then(Value::as_str) {
            Some(value) => value.to_string(),
            None => return Err(MailManErr::new(400, "Bad requests", "id not found", 1)),
        },
        &mut pool.get().unwrap(),
    ) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Delete cron job",
            Some(Value::String(format!("Delete cron: {}", msg))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

// refresh TimeWheel scheduler
pub fn refresh(
    flush_flag: &web::Data<Mutex<bool>>,
) -> Result<MailManOk<'static, Value>, MailManErr<'static>> {
    let mut i = flush_flag.lock().unwrap();
    *i = true;

    Ok(MailManOk::new(
        200,
        "Refresh success",
        Some(serde_json::json!({
            "status": "ok",
            "time": chrono::Local::now()
        })),
    ))
}
