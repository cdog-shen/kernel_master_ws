use std::sync::Mutex;

use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use share_lib::data_structure::MailManErr;

use crate::{
    model::{cron_job::CronJobInfo, job_log::JobLogInfo},
    service::cron_job,
    util::err_mapping::MailManErrResponser,
};

// POST /api/cron_job/get
pub async fn get_all(
    query: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match cron_job::get_all(query.into_inner(), &pool) {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST /api/cron_job/new
pub async fn new(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let info = CronJobInfo::from_map(map.0).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
    })?;
    let _id = uuid::Uuid::new_v4().to_string();

    let _corn = CronJobInfo {
        id: Some(_id.clone()),
        script: Some(
            info.script
                .clone()
                .ok_or(MailManErrResponser::mapping_from_mme(MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing `script` field.".to_string()),
                    1,
                )))?,
        ),
        frequency: Some(info.frequency.ok_or(MailManErrResponser::mapping_from_mme(
            MailManErr::new(
                400,
                "Bad Request",
                Some("Missing `frequency` field.".to_string()),
                1,
            ),
        ))?),
        times: Some(
            info.times
                .ok_or(MailManErrResponser::mapping_from_mme(MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing `times` field.".to_string()),
                    1,
                )))?,
        ),
        params: Some(
            info.params
                .clone()
                .ok_or(MailManErrResponser::mapping_from_mme(MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing `params` field.".to_string()),
                    1,
                )))?,
        ),
        comment: Some(info.comment.clone().unwrap_or(String::new())),
        is_enable: Some(info.is_enable.unwrap_or(false)),
        launch_at: Some(info.launch_at.ok_or(MailManErrResponser::mapping_from_mme(
            MailManErr::new(
                400,
                "Bad Request",
                Some("Missing `launch_at` field.".to_string()),
                1,
            ),
        ))?),
        update_time: info.update_time,
    };

    let _log = JobLogInfo {
        id: Some(_id.clone()),
        script: Some(
            info.script
                .clone()
                .ok_or(MailManErrResponser::mapping_from_mme(MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing `script` field.".to_string()),
                    1,
                )))?,
        ),
        exec_type: Some("cron".to_string()),
        commander: Some(
            crate::server::GLOBAL_CONFIG
                .read()
                .unwrap()
                .subsys_uuid
                .clone(),
        ),
        worker: Some(String::new()),
        status: Some(0),
        params: Some(
            info.params
                .clone()
                .ok_or(MailManErrResponser::mapping_from_mme(MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing `params` field.".to_string()),
                    1,
                )))?,
        ),
        result: Some("{}".to_string()),
        finish_time: Some(String::new()),
        update_time: info.update_time,
        comment: Some(info.comment.clone().unwrap_or(String::new())),
    };

    match cron_job::new(_corn, _log, &pool) {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST /api/cron_job/update
pub async fn update(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let info = CronJobInfo::from_map(map.0).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
    })?;

    match cron_job::update(info, &pool) {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// DELETE /api/cron_job/delete
pub async fn delete(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let id = map
        .0
        .get("id")
        .ok_or_else(|| {
            MailManErrResponser::mapping_from_mme(MailManErr::new(
                400,
                "Bad Request",
                Some("Can Not find 'id' field".to_string()),
                1,
            ))
        })?
        .as_str()
        .ok_or_else(|| {
            MailManErrResponser::mapping_from_mme(MailManErr::new(
                500,
                "Server Error",
                Some("'id' field CAN NOT into String type".to_string()),
                1,
            ))
        })?
        .to_string();

    match cron_job::delete(id, &pool) {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST /api/cron_job/refresh
pub async fn refresh(
    // data: web::Json<Map<String, Value>>,
    flush_flag: web::Data<Mutex<bool>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match cron_job::refresh(&flush_flag) {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}
