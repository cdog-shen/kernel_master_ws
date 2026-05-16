use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::km::cron_job::*;

pub fn get_all<'a>(
    filter: Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Vec<Value>>, MailManErr<'a, String>> {
    match CronJobModel::get_model_info_with_filter(&filter, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service: All cron job", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: All cron job",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: All cron job",
                Some(msg.1),
                1,
            )),
        },
    }
}

pub fn new_table<'a>(
    data: CronJobInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match CronJobModel::new(&data, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Create cron job",
            Some(Value::String(format!("New line: {msg}"))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: Create cron job",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: Create cron job",
                Some(msg.1),
                1,
            )),
        },
    }
}

pub fn update_table<'a>(
    data: CronJobInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match CronJobModel::update(&data, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Update cron job",
            Some(Value::String(format!("Update line: {msg}"))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: Update cron job",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: Update cron job",
                Some(msg.1),
                1,
            )),
        },
    }
}

pub fn delete_table<'a>(
    id: String,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    match CronJobModel::delete(&id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Delete cron job",
            Some(Value::String(format!("Delete line: {msg}"))),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Service: Delete cron job",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                400,
                "Service: Delete cron job",
                Some(msg.1),
                1,
            )),
        },
    }
}
