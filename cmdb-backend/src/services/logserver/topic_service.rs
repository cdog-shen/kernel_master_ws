use actix_web::web;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::{Map, Value};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::models::logservice::topic::*;

// get all
pub fn get_all<'a>(
    filter: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Vec<Value>>, MailManErr<'a>> {
    match LogServiceTopicModel::get_model_info_with_filter(filter.clone(), &mut pool.get().unwrap())
    {
        Ok(msg) => Ok(MailManOk::new(200, "All LogService Topic Info", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

// create new
pub fn new_table<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a>> {
    match LogServiceTopicModel::new(data.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "New LogService Topic Info",
            Some(Value::String(format!("New line: {}", msg))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

// update
pub fn update_table<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a>> {
    match LogServiceTopicModel::update(data.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Update LogService Topic Info",
            Some(Value::String(format!("Update line: {}", msg))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

// delete
pub fn delete_table<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a>> {
    match LogServiceTopicModel::delete(data.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Delete LogService Topic Info",
            Some(Value::String(format!("Delete line: {}", msg))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}
