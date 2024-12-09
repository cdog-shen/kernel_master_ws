use actix_web::{web, HttpResponse};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};

use crate::{
    models::group::GroupInputStream,
    services::group_service,
    utils::err_mapping::MailManErrResponser,
};

// GET api/access_control/all_group
pub async fn all_group(
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match group_service::all_group(&pool) {
        Ok(group_data) => Ok(HttpResponse::Ok().json(group_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/access_control/new_group
pub async fn new_group(
    group_info: web::Json<GroupInputStream>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match group_service::new_group(&group_info, &pool) {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/access_control/update_group
pub async fn update_group(
    group_info: web::Json<GroupInputStream>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match group_service::update_group(&group_info.0, &pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// DEL api/access_control/delete_group
pub async fn delete_group(
    group_id: web::Json<GroupInputStream>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match group_service::delete_group(group_id.0.id.unwrap(), &pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}
