use actix_web::{web, HttpResponse};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::{Map, Value};

// use share_lib::data_structure::MailManOk;

use crate::services::instance::light_ecs_service;

// POST api/cmdb/get_all_table
// pub async fn get_all_table() -> Result<HttpResponse, actix_web::Error> {
//     let data = serde_json::json!(["light_ecs",]);
//     Ok(HttpResponse::Ok().json(data))
// }

// POST api/{table}/get
pub async fn get_table(
    table_name: web::Path<String>,
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let table_name = table_name.into_inner();
    let data = data.into_inner();

    match table_name.as_str() {
        "light_ecs" => {
            let result = light_ecs_service::get_all(&data, &pool);
            match result {
                Ok(data) => return Ok(HttpResponse::Ok().json(data)),
                Err(err) => return Ok(HttpResponse::BadRequest().json(err)),
            }
        }
        _ => Ok(HttpResponse::BadRequest().json("Table not found")),
    }
}

// POST api/{table}/new
pub async fn new_table(
    table_name: web::Path<String>,
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let table_name = table_name.into_inner();
    let data = data.into_inner();

    match table_name.as_str() {
        "light_ecs" => {
            let result = light_ecs_service::new_table(&data, &pool);
            match result {
                Ok(data) => return Ok(HttpResponse::Ok().json(data)),
                Err(err) => return Ok(HttpResponse::InternalServerError().json(err)),
            }
        }
        _ => Ok(HttpResponse::BadRequest().json("Table not found")),
    }
}

// POST api/{table}/update
pub async fn update_table(
    table_name: web::Path<String>,
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let table_name = table_name.into_inner();
    let data = data.into_inner();

    match table_name.as_str() {
        "light_ecs" => {
            let result = light_ecs_service::update_table(&data, &pool);
            match result {
                Ok(data) => return Ok(HttpResponse::Ok().json(data)),
                Err(err) => return Ok(HttpResponse::InternalServerError().json(err)),
            }
        }
        _ => Ok(HttpResponse::BadRequest().json("Table not found")),
    }
}

// POST api/{table}/delete
pub async fn delete_table(
    table_name: web::Path<String>,
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let table_name = table_name.into_inner();
    let data = data.into_inner();

    match table_name.as_str() {
        "light_ecs" => {
            let result = light_ecs_service::delete_table(&data, &pool);
            match result {
                Ok(data) => return Ok(HttpResponse::Ok().json(data)),
                Err(err) => return Ok(HttpResponse::InternalServerError().json(err)),
            }
        }
        _ => Ok(HttpResponse::BadRequest().json("Table not found")),
    }
}
