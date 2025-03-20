use actix_web::{web, HttpResponse};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::{Map, Value};

use crate::services::instance::light_ecs_service;

// GET api/cmdb/get_all_table
// pub async fn get_all_table() -> Result<HttpResponse, actix_web::Error> {
//     let data = serde_json::json!(["light_ecs",]);
//     Ok(HttpResponse::Ok().json(data))
// }

// POST api/table/get
pub async fn get_table(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let data = data.into_inner();
    let table_name = data.get("table").unwrap().as_str().unwrap();
    let query = serde_json::from_value(data.get("query").unwrap().clone()).unwrap();

    match table_name {
        "light_ecs" => {
            let result = light_ecs_service::get_all(&query, &pool);
            match result {
                Ok(data) => return Ok(HttpResponse::Ok().json(data)),
                Err(err) => return Ok(HttpResponse::BadRequest().json(err)),
            }
        }
        _ => Ok(HttpResponse::BadRequest().json("Table not found")),
    }
}

// POST api/table/new
pub async fn new_table(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let data = data.into_inner();
    let table_name = data.get("table").unwrap().as_str().unwrap();
    let new = serde_json::from_value(data.get("new").unwrap().clone()).unwrap();

    match table_name {
        "light_ecs" => {
            let result = light_ecs_service::new_table(&new, &pool);
            match result {
                Ok(data) => return Ok(HttpResponse::Ok().json(data)),
                Err(err) => return Ok(HttpResponse::InternalServerError().json(err)),
            }
        }
        _ => Ok(HttpResponse::BadRequest().json("Table not found")),
    }
}

// POST api/table/update
pub async fn update_table(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let data = data.into_inner();
    let table_name = data.get("table").unwrap().as_str().unwrap();
    let update = serde_json::from_value(data.get("update").unwrap().clone()).unwrap();

    match table_name {
        "light_ecs" => {
            let result = light_ecs_service::update_table(&update, &pool);
            match result {
                Ok(data) => return Ok(HttpResponse::Ok().json(data)),
                Err(err) => return Ok(HttpResponse::InternalServerError().json(err)),
            }
        }
        _ => Ok(HttpResponse::BadRequest().json("Table not found")),
    }
}

// DELETE api/table/delete
pub async fn delete_table(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let data = data.into_inner();
    let table_name = data.get("table").unwrap().as_str().unwrap();
    let delete = serde_json::from_value(data.get("delete").unwrap().clone()).unwrap();

    match table_name {
        "light_ecs" => {
            let result = light_ecs_service::delete_table(&delete, &pool);
            match result {
                Ok(data) => return Ok(HttpResponse::Ok().json(data)),
                Err(err) => return Ok(HttpResponse::InternalServerError().json(err)),
            }
        }
        _ => Ok(HttpResponse::BadRequest().json("Table not found")),
    }
}
