use actix_web::web;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::{Map, Value};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::models::cloudserver::instance::*;

pub fn get_all<'a>(
    filter: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Vec<Value>>, MailManErr<'a, String>> {
    match CloudserverInstanceModel::get_model_info_with_filter(
        filter.clone(),
        &mut pool.get().unwrap(),
    ) {
        Ok(msg) => Ok(MailManOk::new(200, "All Cloudserver info", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", Some(msg.1), 1)),
            _ => Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    }
}

// create new light ecs info
pub fn new_table<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match CloudserverInstanceModel::new(data.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "New Cloudserver info",
            Some(Value::String(format!("New line: {}", msg))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", Some(msg.1), 1)),
            _ => Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    }
}

// update light ecs info
pub fn update_table<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match CloudserverInstanceModel::update(data.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Update Cloudserver info",
            Some(Value::String(format!("Update line: {}", msg))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", Some(msg.1), 1)),
            _ => Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    }
}

// delete light ecs info
pub fn delete_table<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match CloudserverInstanceModel::delete(data.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Delete Cloudserver info",
            Some(Value::String(format!("Delete line: {}", msg))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", Some(msg.1), 1)),
            _ => Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    }
}
