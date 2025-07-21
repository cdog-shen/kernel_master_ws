use crossbeam::queue::SegQueue;
use uuid::Uuid;
use actix_web::{web, HttpResponse};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::{Map, Value};

use crate::services::job_log;

// GET /api/job_log/get
pub async fn get_all(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> HttpResponse {
    match job_log::get_all(&data, &pool) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::BadRequest().json(err),
    }
}

// POST /api/job_log/new
pub async fn new(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> HttpResponse {
    match job_log::new(&data, &pool) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

// POST /api/job_log/update
pub async fn update(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
    done_task_list: web::Data<SegQueue<Uuid>>,
) -> HttpResponse {
    match job_log::update(&data, &pool, &done_task_list) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

// DELETE /api/job_log/delete
pub async fn delete(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> HttpResponse {
    match job_log::delete(&data, &pool) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}
