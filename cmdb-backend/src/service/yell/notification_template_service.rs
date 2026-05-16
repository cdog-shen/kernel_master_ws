use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::yell::notification_template::*;

pub fn get_all<'a>(
    filter: Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Vec<Value>>, MailManErr<'a, String>> {
    match NotificationTemplateModel::get_model_info_with_filter(&filter, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service: All notification_template", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: All notification_template",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: All notification_template",
                Some(msg.1),
                1,
            )),
        },
    }
}

// create new notification_template info
pub fn new_table<'a>(
    data: NotificationTemplateInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match NotificationTemplateModel::new(&data, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Create notification_template",
            Some(Value::String(format!("New line: {msg}"))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: Create notification_template",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: Create notification_template",
                Some(msg.1),
                1,
            )),
        },
    }
}

// update notification_template info
pub fn update_table<'a>(
    data: NotificationTemplateInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match NotificationTemplateModel::update(&data, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Update notification_template",
            Some(Value::String(format!("Update line: {msg}"))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: Update notification_template",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: Update notification_template",
                Some(msg.1),
                1,
            )),
        },
    }
}

// delete notification_template info
pub fn delete_table<'a>(
    id: i32,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match NotificationTemplateModel::delete(id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Delete notification_template",
            Some(Value::String(format!("Delete line: {msg}"))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: Delete notification_template",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: Delete notification_template",
                Some(msg.1),
                1,
            )),
        },
    }
}
