use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::yell::channel_config::*;

pub fn get_all<'a>(
    filter: Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Vec<Value>>, MailManErr<'a, String>> {
    match ChannelConfigModel::get_model_info_with_filter(&filter, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service: All channel_config", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: All channel_config",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: All channel_config",
                Some(msg.1),
                1,
            )),
        },
    }
}

// create new channel_config info
pub fn new_table<'a>(
    data: ChannelConfigInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match ChannelConfigModel::new(&data, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Create channel_config",
            Some(Value::String(format!("New line: {msg}"))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: Create channel_config",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: Create channel_config",
                Some(msg.1),
                1,
            )),
        },
    }
}

// update channel_config info
pub fn update_table<'a>(
    data: ChannelConfigInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match ChannelConfigModel::update(&data, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Update channel_config",
            Some(Value::String(format!("Update line: {msg}"))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: Update channel_config",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: Update channel_config",
                Some(msg.1),
                1,
            )),
        },
    }
}

// delete channel_config info
pub fn delete_table<'a>(
    id: i32,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match ChannelConfigModel::delete(id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Delete channel_config",
            Some(Value::String(format!("Delete line: {msg}"))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: Delete channel_config",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: Delete channel_config",
                Some(msg.1),
                1,
            )),
        },
    }
}
