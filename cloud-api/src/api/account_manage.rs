use actix_web::{web, HttpResponse};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::{Map, Value};

use crate::services::account_service;

// GET /api/account_db/get
pub async fn get_all(
    query: web::Query<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> HttpResponse {
    match account_service::get_all(&query, &pool) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

// POST /api/account_db/new
pub async fn new(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> HttpResponse {
    match account_service::new(&data, &pool) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

// POST /api/account_db/update
pub async fn update(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> HttpResponse {
    match account_service::update(&data, &pool) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

// DELETE /api/account_db/delete
pub async fn delete(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> HttpResponse {
    match account_service::delete(&data, &pool) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}
