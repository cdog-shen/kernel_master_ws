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

// POST api/{operation}/{db}
pub async fn db_operation(
    operation: web::Path<String>,
    db: web::Path<String>,
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let operation = operation.into_inner();
    let db = db.into_inner();
    let data = data.into_inner();

    match operation.as_str() {
        "get" => get_table(db.into(), data, pool).await,
        "new" => new_table(db.into(), data, pool).await,
        "update" => update_table(db.into(), data, pool).await,
        "delete" => delete_table(db.into(), data, pool).await,
        _ => Ok(HttpResponse::BadRequest().json("Operation not found")),
    }
}

async fn get_table(
    table_name: String,
    data: Map<String, Value>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, actix_web::Error> {
    match &table_name[..] {
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

async fn new_table(
    table_name: String,
    data: Map<String, Value>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, actix_web::Error> {
    match &table_name[..] {
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

async fn update_table(
    table_name: String,
    data: Map<String, Value>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, actix_web::Error> {
    match &table_name[..] {
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

pub async fn delete_table(
    table_name: String,
    data: Map<String, Value>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, actix_web::Error> {
    match &table_name[..] {
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
