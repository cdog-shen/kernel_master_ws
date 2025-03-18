use actix_web::{web, HttpResponse};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::{Map, Value};

use crate::{
    model::access::AccessInputStream, service::access_service,
    util::err_mapping::MailManErrResponser,
};

// GET api/access_control/all_access
pub async fn all_access(
    query: web::Query<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match access_service::all_access(&query, &pool) {
        Ok(access_data) => Ok(HttpResponse::Ok().json(access_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/access_control/new_access
pub async fn new_access(
    access_info: web::Json<AccessInputStream>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match access_service::new_access(&access_info, &pool) {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/access_control/update_access
pub async fn update_access(
    access_info: web::Json<AccessInputStream>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match access_service::update_access(&access_info.0, &pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// DEL api/access_control/delete_access
pub async fn delete_access(
    access_id: web::Json<AccessInputStream>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match access_service::delete_access(access_id.0.id.unwrap(), &pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}
