use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::group::*;

/// all_group api logic
pub fn all_group<'a>(
    filter: Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Vec<GroupModel>>, MailManErr<'a, String>> {
    match GroupModel::get_all_with_filter(&filter, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service: All group", Some(msg))),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(400, "Service: All group", Some(msg.1), 1)),
            _ => Err(MailManErr::new(500, "Service: All group", Some(msg.1), 1)),
        },
    }
}

/// new_group api logic
pub fn new_group<'a>(
    group_info: GroupInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match GroupModel::new_group(&group_info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Servce: Create group",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(
                400,
                "Service: Create group",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                500,
                "Service: Create group",
                Some(msg.1),
                1,
            )),
        },
    }
}

/// update_group api logic
pub fn update_group<'a>(
    group_info: GroupInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match GroupModel::update_group_by_id(&group_info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Servce: Update group",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(
                400,
                "Service: Update group",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                500,
                "Service: Update group",
                Some(msg.1),
                1,
            )),
        },
    }
}

/// delete_group api logic
pub fn delete_group<'a>(
    id: i32,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match GroupModel::delete_group_by_id(id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Servce: Delete group",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(
                400,
                "Service: Delete group",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                500,
                "Service: Delete group",
                Some(msg.1),
                1,
            )),
        },
    }
}
