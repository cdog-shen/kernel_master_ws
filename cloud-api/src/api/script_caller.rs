use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::Value;
use share_lib::data_structure::MailManErr;
use share_lib::err_mapping::MailManErrResponser;

use crate::service::script_caller;

// run cloud api script
pub async fn run(
    req: web::Json<Value>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let bad_request = |field: &str| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(
            400,
            "Bad Request",
            Some(format!("'{field}' field MUST be string")),
            1,
        ))
    };

    let api_name = req["api_name"]
        .as_str()
        .ok_or_else(|| bad_request("api_name"))?;
    let cloud_user = req["cloud_user"]
        .as_str()
        .ok_or_else(|| bad_request("cloud_user"))?;
    let region = req["region"]
        .as_str()
        .ok_or_else(|| bad_request("region"))?;
    let params = req["params"]
        .as_str()
        .ok_or_else(|| bad_request("params"))?;

    match script_caller::run(api_name, cloud_user, region, params, &pool).await {
        Ok(res_data) => Ok(HttpResponse::Ok().json(res_data)),
        Err(err_data) => Err(MailManErrResponser::mapping_from_mme(err_data)),
    }
}

// get cloud api script
pub async fn get(req: web::Json<Value>) -> Result<HttpResponse, MailManErrResponser> {
    let bad_request = |field: &str| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(
            400,
            "Bad Request",
            Some(format!("'{field}' field MUST be string")),
            1,
        ))
    };

    let provider_name = req["provider_name"]
        .as_str()
        .ok_or_else(|| bad_request("provider_name"))?;
    let product_name = req["product_name"]
        .as_str()
        .ok_or_else(|| bad_request("product_name"))?;

    match script_caller::get_scripts(provider_name, product_name).await {
        Ok(res_data) => Ok(HttpResponse::Ok().json(res_data)),
        Err(err_data) => Err(MailManErrResponser::mapping_from_mme(err_data)),
    }
}
