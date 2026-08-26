use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use share_lib::data_structure::MailManErr;
use share_lib::err_mapping::MailManErrResponser;

use crate::{model::group, service::group_service};

// GET api/group_control/all_group
pub async fn all_group(
    query: web::Query<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match group_service::all_group(query.0, &pool).await {
        Ok(group_data) => Ok(HttpResponse::Ok().json(group_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/group_control/new_group
pub async fn new_group(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let group_info = group::GroupInfo::from_map(map.0).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
    })?;

    let group_info = group::GroupInfo {
        id: None,
        group_name: Some(group_info.group_name.clone().ok_or(
            MailManErrResponser::mapping_from_mme(MailManErr::new(
                400,
                "Bad Request",
                Some("Missing `group_name` field.".to_string()),
                1,
            )),
        )?),
        is_enable: Some(group_info.is_enable.unwrap_or(false)),
        update_time: group_info.update_time,
        user_ids: Some(group_info.user_ids.clone().unwrap_or(serde_json::json!([]))),
    };

    match group_service::new_group(group_info, &pool).await {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/group_control/update_group
pub async fn update_group(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let group_info = group::GroupInfo::from_map(map.0).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
    })?;

    match group_service::update_group(group_info, &pool).await {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// DEL api/group_control/delete_group
pub async fn delete_group(
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

    match group_service::delete_group(id, &pool).await {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}
