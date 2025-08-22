use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
// use serde::{Deserialize, Serialize};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::access::*;

/// all_access api logic
pub fn all_access<'a>(
    filter: Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Vec<AccessModel>>, MailManErr<'a, String>> {
    match AccessModel::get_all_with_filter(&filter, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service: All access", Some(msg))),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(400, "Service: All access", Some(msg.1), 1)),
            _ => Err(MailManErr::new(500, "Service: All access", Some(msg.1), 1)),
        },
    }
}

/// new_access api logic
pub fn new_access<'a>(
    info: AccessInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match AccessModel::new_access(&info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Create access",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(
                400,
                "Service: Create access",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                500,
                "Service: Create access",
                Some(msg.1),
                1,
            )),
        },
    }
}

/// update_access api logic
pub fn update_access<'a>(
    info: AccessInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match AccessModel::update_access_by_id(&info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Update access",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(
                400,
                "Service: Update access",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                500,
                "Service: Update access",
                Some(msg.1),
                1,
            )),
        },
    }
}

/// delete_access api logic
pub fn delete_access<'a>(
    id: i32,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match AccessModel::delete_access_by_id(id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Delete access",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(
                400,
                "Service: Delete access",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                500,
                "Service: Delete access",
                Some(msg.1),
                1,
            )),
        },
    }
}
