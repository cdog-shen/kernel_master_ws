use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::watchman::group::*;

pub fn get_all<'a>(
    filter: Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Vec<Value>>, MailManErr<'a, String>> {
    match GroupModel::get_model_info_with_filter(&filter, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service: All group", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: All group",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: All group",
                Some(msg.1),
                1,
            )),
        },
    }
}

pub fn new_table<'a>(
    data: GroupInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match GroupModel::new(&data, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Create group",
            Some(Value::String(format!("New line: {msg}"))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: Create group",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: Create group",
                Some(msg.1),
                1,
            )),
        },
    }
}

pub fn update_table<'a>(
    data: GroupInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match GroupModel::update(&data, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Update group",
            Some(Value::String(format!("Update line: {msg}"))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: Update group",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: Update group",
                Some(msg.1),
                1,
            )),
        },
    }
}

pub fn delete_table<'a>(
    id: i32,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match GroupModel::delete(id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Delete group",
            Some(Value::String(format!("Delete line: {msg}"))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: Delete group",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: Delete group",
                Some(msg.1),
                1,
            )),
        },
    }
}
