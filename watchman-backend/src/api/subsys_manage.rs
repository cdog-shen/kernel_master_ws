use actix_web::{web, HttpResponse};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};

use crate::{
    models::subsys::SubsysInputStream, services::subsys_service,
    utils::err_mapping::MailManErrResponser,
};

// GET api/subsystem_control/all_subsystem
pub async fn all_subsys(
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match subsys_service::all_subsys(&pool) {
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
