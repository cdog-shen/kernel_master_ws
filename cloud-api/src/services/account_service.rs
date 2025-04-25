use actix_web::web;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::{Map, Value};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::models::cloud_account::*;

// get all cloud account info
pub fn get_all<'a>(
    filter: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Vec<Value>>, MailManErr<'a>> {
    match CloudAccountModel::get_user_info_with_filter(filter.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "All cloud account", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

// create new cloud account
pub fn new<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a>> {
    match CloudAccountModel::new_account(data.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "New cloud account",
            Some(Value::String(format!("New line: {}", msg))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

// update cloud account info
pub fn update<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a>> {
    match CloudAccountModel::update_account(data.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Update cloud account",
            Some(Value::String(format!("Update line: {}", msg))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

// delete cloud account info
pub fn delete<'a>(
    data: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a>> {
    match CloudAccountModel::delete_account(
        match data.get("id").and_then(Value::as_u64) {
            Some(value) => value as u32,
            None => return Err(MailManErr::new(400, "Bad requests", "id not found", 1)),
        },
        &mut pool.get().unwrap(),
    ) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Delete cloud account",
            Some(Value::String(format!("Delete line: {}", msg))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}
