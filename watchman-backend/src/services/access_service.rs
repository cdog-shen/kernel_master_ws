use actix_web::web;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
// use serde::{Deserialize, Serialize};
// use serde_json::json;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::models::group::{GroupInfo, GroupModel};

/// all_group api logic
pub fn all_group<'a>(
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Vec<GroupModel>>, MailManErr<'a>> {
    match GroupModel::get_all(&mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "info updated", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

/// new_group api logic
pub fn new_group<'a>(
    group_info: &GroupInfo,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    match GroupModel::new_group(&group_info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "info updated", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

/// update_group api logic
pub fn update_group<'a>(
    user_info: &GroupInfo,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    match GroupModel::update_group_by_id(&user_info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "info updated", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

/// delete_group api logic
pub fn delete_group<'a>(
    group_id: u32,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    match GroupModel::delete_group_by_id(group_id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "info updated", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}
