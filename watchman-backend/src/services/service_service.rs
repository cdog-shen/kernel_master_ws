use actix_web::web;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
// use serde::{Deserialize, Serialize};
// use serde_json::json;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::models::service::*;

/// all_service api logic
pub fn all_service<'a>(
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Vec<ServiceOutputStream>>, MailManErr<'a>> {
    match ServiceModel::get_all(&mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "All service info", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

/// new_service api logic
pub fn new_service<'a>(
    service_info: &ServiceInputStream,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    match ServiceModel::new_service(&service_info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service created", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

/// update_service api logic
pub fn update_service<'a>(
    service_info: &ServiceInputStream,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    match ServiceModel::update_service_by_id(&service_info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service info updated", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

/// delete_service api logic
pub fn delete_service<'a>(
    service_id: u32,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    match ServiceModel::delete_service_by_id(service_id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service deleted", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}
