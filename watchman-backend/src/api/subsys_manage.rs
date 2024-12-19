use actix_web::{web, HttpResponse};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::{Map, Value};

use crate::{
    models::subsys::SubsysInputStream, services::subsys_service,
    utils::err_mapping::MailManErrResponser,
};

// GET api/subsystem_control/all_subsystem
pub async fn all_subsys(
    query: web::Query<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match subsys_service::all_subsys(&query, &pool) {
        Ok(subsys_data) => Ok(HttpResponse::Ok().json(subsys_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/subsystem_control/new_subsystem
pub async fn new_subsys(
    subsys_meta: web::Json<SubsysInputStream>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match subsys_service::new_subsys(&subsys_meta, &pool) {
        Ok(subsys_data) => Ok(HttpResponse::Ok().json(subsys_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/subsystem_control/update_subsystem
pub async fn update_subsys(
    subsys_meta: web::Json<SubsysInputStream>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match subsys_service::update_subsys(&subsys_meta, &pool) {
        Ok(subsys_data) => Ok(HttpResponse::Ok().json(subsys_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// DELETE api/subsystem_control/delete_subsystem
pub async fn delete_subsys(
    subsys_meta: web::Json<SubsysInputStream>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match subsys_service::delete_subsys(subsys_meta.0.id.unwrap(), &pool) {
        Ok(subsys_data) => Ok(HttpResponse::Ok().json(subsys_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

/// POST api/subsystem_call/call_subsystem
pub async fn call_subsys(
    subsys_name: web::Path<String>,
    subsys_params: web::Json<serde_json::Map<String, serde_json::Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match subsys_service::call((*subsys_name.clone()).to_string(), subsys_params.0, &pool) {
        Ok(subsys_data) => Ok(HttpResponse::Ok().json(subsys_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}
