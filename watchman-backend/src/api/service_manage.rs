use actix_web::{web, HttpResponse};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};

use crate::{
    models::service::ServiceInputStream,
    services::service_service,
    utils::err_mapping::MailManErrResponser,
};


// GET api/service_control/all_service
pub async fn all_service(
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match service_service::all_service(&pool) {
        Ok(group_data) => Ok(HttpResponse::Ok().json(group_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/service_control/new_service
pub async fn new_service(
    group_info: web::Json<ServiceInputStream>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match service_service::new_service(&group_info, &pool) {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/service_control/update_service
pub async fn update_service(
    user_info: web::Json<ServiceInputStream>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match service_service::update_service(&user_info.0, &pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// DEL api/service_control/delete_service
pub async fn delete_service(
    group_id: web::Json<ServiceInputStream>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match service_service::delete_service(group_id.0.id.unwrap(), &pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}
