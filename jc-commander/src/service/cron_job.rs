use std::sync::Mutex;

use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::{
    cron_job::*,
    job_log::{JobLogInfo, JobLogModel},
};

// get cron by filter
pub fn get_all<'a>(
    filter: Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Vec<CronJobModel>>, MailManErr<'a, String>> {
    match CronJobModel::get_crons_with_filter(&filter, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service: All Cron", Some(msg))),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(400, "Service: All Cron", Some(msg.1), 1)),
            _ => Err(MailManErr::new(500, "Service: All Cron", Some(msg.1), 1)),
        },
    }
}

// new cron job
pub fn new<'a>(
    cron: CronJobInfo,
    log: JobLogInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match CronJobModel::new_cron(&cron, &mut pool.get().unwrap()) {
        Ok(msg) => {
            MailManOk::new(
                200,
                "Service: New Cron",
                Some(format!("Line changed: {msg}")),
            );
        }
        Err(msg) => match msg.0 {
            1 => return Err(MailManErr::new(400, "Service: New Cron", Some(msg.1), 1)),
            _ => return Err(MailManErr::new(500, "Service: New Cron", Some(msg.1), 1)),
        },
    }

    match JobLogModel::new_log(&log, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: New Cron",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(
                400,
                "Service: New Cron (log)",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(
                500,
                "Service: New Cron (log)",
                Some(msg.1),
                1,
            )),
        },
    }
}

// update cron job
pub fn update<'a>(
    cron: CronJobInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match CronJobModel::update_cron(&cron, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Update Cron",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(400, "Service: Update Cron", Some(msg.1), 1)),
            _ => Err(MailManErr::new(500, "Service: Update Cron", Some(msg.1), 1)),
        },
    }
}

// delete job log
pub fn delete<'a>(
    id: String,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match CronJobModel::delete_cron(&id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Delete Cron",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(400, "Service: Delete Cron", Some(msg.1), 1)),
            _ => Err(MailManErr::new(500, "Service: Delete Cron", Some(msg.1), 1)),
        },
    }
}

// refresh TimeWheel scheduler
pub fn refresh(
    flush_flag: &web::Data<Mutex<bool>>,
) -> Result<MailManOk<'static, Value>, MailManErr<'static, String>> {
    let mut i = flush_flag.lock().unwrap();
    *i = true;

    Ok(MailManOk::new(
        200,
        "Service: Refresh Timing Wheel",
        Some(serde_json::json!({
            "status": "ok",
            "time": chrono::Local::now()
        })),
    ))
}
