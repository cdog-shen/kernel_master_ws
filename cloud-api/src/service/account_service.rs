use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::cloud_account::*;

// get all cloud account info
pub fn get_all<'a>(
    filter: Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Vec<Value>>, MailManErr<'a, String>> {
    match CloudAccountModel::get_model_with_filter(&filter, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service: All CloudAccount", Some(msg))),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(
                400,
                "Service: All CloudAccount",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                500,
                "Service: All CloudAccount",
                Some(msg.1),
                1,
            )),
        },
    }
}

// create new cloud account
pub fn new<'a>(
    data: CloudAccountInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match CloudAccountModel::new_account(&data, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Create CloudAccount",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: Create CloudAccount",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: Create CloudAccount",
                Some(msg.1),
                1,
            )),
        },
    }
}

// update cloud account info
pub fn update<'a>(
    data: CloudAccountInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match CloudAccountModel::update_account(&data, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Update CloudAccount",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(
                400,
                "Service: Update CloudAccount",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                500,
                "Service: Update CloudAccount",
                Some(msg.1),
                1,
            )),
        },
    }
}

// delete cloud account info
pub fn delete<'a>(
    id: i32,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match CloudAccountModel::delete_account(id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Delete CloudAccount",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: Delete CloudAccount",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: Delete CloudAccount",
                Some(msg.1),
                1,
            )),
        },
    }
}
