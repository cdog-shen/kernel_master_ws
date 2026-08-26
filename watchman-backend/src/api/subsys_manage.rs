use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use share_lib::data_structure::MailManErr;
use share_lib::err_mapping::MailManErrResponser;

use crate::{model::subsys, service::subsys_service};

// GET api/subsystem_control/all_subsystem
pub async fn all_subsys(
    query: web::Query<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match subsys_service::all_subsys(query.0, &pool) {
        Ok(subsys_data) => Ok(HttpResponse::Ok().json(subsys_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/subsystem_control/new_subsystem
pub async fn new_subsys(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let subsys_info = subsys::SubsysInfo::from_map(map.0).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
    })?;

    let subsys_info = subsys::SubsysInfo {
        id: None,
        subsys_name: Some(subsys_info.subsys_name.clone().ok_or(
            MailManErrResponser::mapping_from_mme(MailManErr::new(
                400,
                "Bad Request",
                Some("Missing `subsys_name` field.".to_string()),
                1,
            )),
        )?),
        url: Some(
            subsys_info
                .url
                .clone()
                .ok_or(MailManErrResponser::mapping_from_mme(MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing `url` field.".to_string()),
                    1,
                )))?,
        ),
        token: Some(
            subsys_info
                .token
                .clone()
                .ok_or(MailManErrResponser::mapping_from_mme(MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing `token` field.".to_string()),
                    1,
                )))?,
        ),
        relate_service_id: Some(subsys_info.relate_service_id.unwrap_or(0)),
        is_enable: Some(subsys_info.is_enable.unwrap_or(false)),
        update_time: subsys_info.update_time,
    };

    match subsys_service::new_subsys(subsys_info, &pool) {
        Ok(subsys_data) => Ok(HttpResponse::Ok().json(subsys_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/subsystem_control/update_subsystem
pub async fn update_subsys(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let subsys_info = subsys::SubsysInfo::from_map(map.0).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
    })?;

    match subsys_service::update_subsys(subsys_info, &pool) {
        Ok(subsys_data) => Ok(HttpResponse::Ok().json(subsys_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// DELETE api/subsystem_control/delete_subsystem
pub async fn delete_subsys(
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
        .as_i64()
        .ok_or_else(|| {
            MailManErrResponser::mapping_from_mme(MailManErr::new(
                400,
                "Bad Request",
                Some("'id' field MUST be i32".to_string()),
                1,
            ))
        })? as i32;

    match subsys_service::delete_subsys(id, &pool) {
        Ok(subsys_data) => Ok(HttpResponse::Ok().json(subsys_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

/// POST api/subsystem_call/call_subsystem
pub async fn call_subsys(
    subsys_name: web::Path<String>,
    subsys_params: web::Json<serde_json::Map<String, serde_json::Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match subsys_service::call((*subsys_name.clone()).to_string(), subsys_params.0, &pool) {
        Ok(subsys_data) => Ok(HttpResponse::Ok().json(subsys_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}
