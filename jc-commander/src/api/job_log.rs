use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use share_lib::data_structure::MailManErr;
use share_lib::err_mapping::MailManErrResponser;

use crate::{api::filter, model::job_log::JobLogInfo, service::job_log};

// POST /api/job_log/get
pub async fn get_all(
    query: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match job_log::get_all(filter::clean_job_log_filter(query.into_inner()), &pool).await {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST /api/job_log/new
// pub async fn new(
//     map: web::Json<Map<String, Value>>,
//     pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
// ) -> Result<HttpResponse, MailManErrResponser> {
//     let info = JobLogInfo::from_map(map.0).map_err(|e| {
//         MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
//     })?;

//     match job_log::new(info, &pool) {
//         Ok(data) => Ok(HttpResponse::Ok().json(data)),
//         Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
//     }
// }

// POST /api/job_log/update
pub async fn update(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let info = JobLogInfo::from_map(map.0).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
    })?;

    match job_log::update(info, &pool).await {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// DELETE /api/job_log/delete
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

    match job_log::delete(id, &pool).await {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}
