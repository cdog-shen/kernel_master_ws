use actix_web::web;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::{Map, Value};
// use serde::{Deserialize, Serialize};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::access::*;

/// all_access api logic
pub fn all_access<'a>(
    filter: &'a Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Vec<AccessOutputStream>>, MailManErr<'a>> {
    match AccessModel::get_all_with_filter(filter.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "All access info", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

/// new_access api logic
pub fn new_access<'a>(
    service_info: &AccessInputStream,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    match AccessModel::new_access(service_info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Access created", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

/// update_access api logic
pub fn update_access<'a>(
    service_info: &AccessInputStream,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    match AccessModel::update_access_by_id(service_info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Access info updated", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

/// delete_access api logic
pub fn delete_access<'a>(
    service_id: u32,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    match AccessModel::delete_access_by_id(service_id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Access deleted", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}
