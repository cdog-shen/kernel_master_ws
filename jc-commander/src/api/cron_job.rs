use std::sync::Mutex;

use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use share_lib::data_structure::MailManErr;
use share_lib::err_mapping::MailManErrResponser;

use crate::{api::filter, model::cron_job::CronJobInfo, service::cron_job};

// POST /api/cron_job/get
pub async fn get_all(
    query: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match cron_job::get_all(filter::clean_cron_job_filter(query.into_inner()), &pool).await {
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

    // 必填字段清洗：缺失返回 400；uuid/commander/双实体组装在 service 层完成
    let miss = |field: &str| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(
            400,
            "Bad Request",
            Some(format!("Missing `{field}` field.")),
            1,
        ))
    };
    if info.script.is_none() {
        return Err(miss("script"));
    }
    if info.frequency.is_none() {
        return Err(miss("frequency"));
    }
    if info.times.is_none() {
        return Err(miss("times"));
    }
    if info.params.is_none() {
        return Err(miss("params"));
    }
    if info.launch_at.is_none() {
        return Err(miss("launch_at"));
    }

    match cron_job::new(info, &pool).await {
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

    match cron_job::update(info, &pool).await {
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

    match cron_job::delete(id, &pool).await {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST /api/cron_job/refresh
pub async fn refresh(
    // data: web::Json<Map<String, Value>>,
    flush_flag: web::Data<Mutex<bool>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match cron_job::refresh(&flush_flag).await {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}
