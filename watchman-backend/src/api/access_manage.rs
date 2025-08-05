use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use share_lib::data_structure::MailManErr;

use crate::{model::access, service::access_service, util::err_mapping::MailManErrResponser};

// GET api/access_control/all_access
pub async fn all_access(
    query: web::Query<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match access_service::all_access(query.0, &pool) {
        Ok(access_data) => Ok(HttpResponse::Ok().json(access_data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/access_control/new_access
pub async fn new_access(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let access_info = access::AccessInfo::from_map(map.0).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
    })?;

    let access_info =
        access::AccessInfo {
            id: None,
            service_id: Some(access_info.service_id.ok_or(
                MailManErrResponser::mapping_from_mme(MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing `service_id` field.".to_string()),
                    1,
                )),
            )?),
            group_id: Some(
                access_info
                    .group_id
                    .ok_or(MailManErrResponser::mapping_from_mme(MailManErr::new(
                        400,
                        "Bad Request",
                        Some("Missing `group_id` field.".to_string()),
                        1,
                    )))?,
            ),
            group_access: Some(access_info.group_access.unwrap_or(0)),
            is_enable: Some(access_info.is_enable.unwrap_or(false)),
            comment: Some(access_info.comment.clone().unwrap_or(String::new())),
            update_time: access_info.update_time,
        };

    match access_service::new_access(access_info, &pool) {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/access_control/update_access
pub async fn update_access(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let access_info = access::AccessInfo::from_map(map.0).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
    })?;

    match access_service::update_access(access_info, &pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// DEL api/access_control/delete_access
pub async fn delete_access(
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

    match access_service::delete_access(id, &pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}
