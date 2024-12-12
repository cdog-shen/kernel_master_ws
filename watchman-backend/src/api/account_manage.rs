use actix_web::{web, HttpResponse};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde::{Deserialize, Serialize};

use crate::{
    models::user::{self, *},
    services::account_service,
    utils::err_mapping::MailManErrResponser,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct InputJsonStruct {
    pub id: Option<u32>,
    pub username: Option<String>,
    pub passwd: Option<String>,
    pub is_enable: Option<u8>,
    pub name: Option<String>,
    pub contact: Option<serde_json::Value>,
    pub date_joined: Option<chrono::NaiveDateTime>,
    pub last_login: Option<chrono::NaiveDateTime>,
}

fn deserialization_input_json_struct(input: InputJsonStruct) -> UserInputStream {
    UserInputStream {
        id: input.id,
        username: input.username,
        passwd: input.passwd,
        is_enable: input.is_enable,
        name: input.name,
        contact: Some(serde_json::to_string(&input.contact.unwrap()).unwrap()),
        date_joined: None,
        last_login: None,
    }
}

// GET api/auth/all_user
pub async fn get_all(
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match account_service::get_all(&pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/auth/login
pub async fn login(
    login_json: web::Json<UserInputStream>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match account_service::login(login_json.0, &pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/auth/signup
pub async fn signup(
    user_basic_info: web::Json<UserInputStream>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match account_service::new_user(user_basic_info.0, &pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/auth/logout
pub async fn logout(
    user_basic_info: web::Json<UserInputStream>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match account_service::logout(user_basic_info.0.username.unwrap(), &pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}

// POST api/auth/user_update
pub async fn user_update(
    user_info: web::Json<InputJsonStruct>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match account_service::user_update(deserialization_input_json_struct(user_info.0), &pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}
