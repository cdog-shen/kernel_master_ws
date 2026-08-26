use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use share_lib::data_structure::MailManErr;
use share_lib::err_mapping::MailManErrResponser;

use crate::{api::filter, model::user, service::account_service};

// GET api/auth/me/{id}
pub async fn get_me(
    user_id: web::Path<i32>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match account_service::get_me(*user_id, &pool).await {
        Ok(user_full_info) => Ok(HttpResponse::Ok().json(user_full_info)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// GET api/auth/all_user
pub async fn all_user(
    query: web::Query<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match account_service::get_all(filter::clean_user_filter(query.0), &pool).await {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/auth/login
pub async fn login(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let user_info = user::UserInputStream::from_map(map.0).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
    })?;

    match account_service::login(user_info, &pool).await {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/auth/signup
pub async fn signup(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let user_info = user::UserInputStream::from_map(map.0).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
    })?;

    let user_info =
        user::UserInputStream {
            id: None,
            username: Some(user_info.username.clone().ok_or(
                MailManErrResponser::mapping_from_mme(MailManErr::new(
                    400,
                    "Bad Request",
                    Some("Missing `username` field.".to_string()),
                    1,
                )),
            )?),
            passwd: Some(
                user_info
                    .passwd
                    .clone()
                    .ok_or(MailManErrResponser::mapping_from_mme(MailManErr::new(
                        400,
                        "Bad Request",
                        Some("Missing `passwd` field.".to_string()),
                        1,
                    )))?,
            ),
            full_name: Some(user_info.full_name.clone().unwrap_or(String::new())),
            contact: Some(user_info.contact.clone().unwrap_or(serde_json::json!({}))),
            is_enable: Some(user_info.is_enable.unwrap_or(false)),
            update_time: user_info.update_time,
        };

    match account_service::new_user(user_info, &pool).await {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/auth/logout
pub async fn logout(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let user_info = user::UserInputStream::from_map(map.0).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
    })?;

    match account_service::logout(user_info, &pool).await {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/auth/user_update
pub async fn user_update(
    map: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let user_info = user::UserInputStream::from_map(map.0).map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
    })?;

    match account_service::user_update(user_info, &pool).await {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}
