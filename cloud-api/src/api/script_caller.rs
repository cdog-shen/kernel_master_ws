use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::Value;
use share_lib::err_mapping::MailManErrResponser;

use crate::service::script_caller;

// run cloud api script
pub async fn run(
    req: web::Json<Value>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let api_name = req["api_name"].as_str().unwrap();
    let cloud_user = req["cloud_user"].as_str().unwrap();
    let region = req["region"].as_str().unwrap();
    let params = req["params"].as_str().unwrap();

    match script_caller::run(api_name, cloud_user, region, params, &pool) {
        Ok(res_data) => Ok(HttpResponse::Ok().json(res_data)),
        Err(err_data) => Err(MailManErrResponser::mapping_from_mme(err_data)),
    }
}

// get cloud api script
pub async fn get(req: web::Json<Value>) -> Result<HttpResponse, MailManErrResponser> {
    let provider_name = req["provider_name"].as_str().unwrap();
    let product_name = req["product_name"].as_str().unwrap();

    match script_caller::get_scripts(provider_name, product_name) {
        Ok(res_data) => Ok(HttpResponse::Ok().json(res_data)),
        Err(err_data) => Err(MailManErrResponser::mapping_from_mme(err_data)),
    }
}
