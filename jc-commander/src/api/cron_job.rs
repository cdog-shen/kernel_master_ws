use std::sync::Mutex;

use actix_web::{web, HttpResponse};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::{Map, Value};

use crate::services::cron_job;

// GET /api/cron_job/get
pub async fn get_all(
    query: web::Query<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> HttpResponse {
    match cron_job::get_all(&query, &pool) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

// POST /api/cron_job/new
pub async fn new(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> HttpResponse {
    match cron_job::new(&data, &pool) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

// POST /api/cron_job/update
pub async fn update(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> HttpResponse {
    match cron_job::update(&data, &pool) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

// DELETE /api/cron_job/delete
pub async fn delete(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> HttpResponse {
    match cron_job::delete(&data, &pool) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

// POST /api/cron_job/refresh
pub async fn refresh(
    // data: web::Json<Map<String, Value>>,
    flush_flag: web::Data<Mutex<bool>>,
) -> HttpResponse {
    match cron_job::refresh(&flush_flag) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}
