use actix_web::{web, HttpResponse};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    models::group::GroupInputStream, services::group_service,
    utils::err_mapping::MailManErrResponser,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct InputJsonStruct {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub is_enable: Option<u8>,
    pub date_update: Option<chrono::NaiveDateTime>,
    pub user_ids: Option<serde_json::Value>,
}

fn deserialization_input_json_struct(input: InputJsonStruct) -> GroupInputStream {
    GroupInputStream {
        id: input.id,
        name: input.name,
        is_enable: input.is_enable,
        user_ids: Some(
            serde_json::to_string(&input.user_ids.unwrap_or(serde_json::json!(""))).unwrap(),
        ),
        date_update: None,
    }
}

// GET api/group_control/all_group
pub async fn all_group(
    query: web::Query<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match group_service::all_group(&query, &pool) {
        Ok(group_data) => Ok(HttpResponse::Ok().json(group_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/group_control/new_group
pub async fn new_group(
    group_info: web::Json<InputJsonStruct>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match group_service::new_group(&deserialization_input_json_struct(group_info.0), &pool) {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/group_control/update_group
pub async fn update_group(
    group_info: web::Json<InputJsonStruct>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match group_service::update_group(&deserialization_input_json_struct(group_info.0), &pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// DEL api/group_control/delete_group
pub async fn delete_group(
    group_id: web::Json<InputJsonStruct>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match group_service::delete_group(group_id.0.id.unwrap(), &pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}
