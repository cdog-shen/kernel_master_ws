use actix_web::web;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::models::{service::*, subsys::*};

/// all_subsys api logic
pub fn all_subsys<'a>(
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Vec<SubsysOutputStream>>, MailManErr<'a>> {
    match SubsysModel::get_all(&mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "All Subsystem info", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

/// new_subsys api logic
pub fn new_subsys<'a>(
    subsys_info: &SubsysInputStream,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    match SubsysModel::new_meta(&subsys_info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Subsystem meta data created",
            Some(msg),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

/// update_subsys api logic
pub fn update_subsys<'a>(
    subsys_info: &SubsysInputStream,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    match SubsysModel::update_meta_by_id(&subsys_info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Subsystem meta data updated",
            Some(msg),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

/// delete_subsys api logic
pub fn delete_subsys<'a>(
    subsys_meta_id: u32,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    match SubsysModel::delete_meta_by_id(subsys_meta_id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Subsystem meta data deleted", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}
