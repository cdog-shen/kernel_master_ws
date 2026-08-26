use actix_web::{HttpResponse, web};
use serde_json::{Map, Value};
use std::path::PathBuf;
// use uuid::Uuid;

use share_lib::err_mapping::MailManErrResponser;

use crate::service::dir_manage;

// POST api/directory/check
pub async fn check(
    data: web::Json<Map<String, Value>>,
    root: web::Data<PathBuf>,
) -> Result<HttpResponse, MailManErrResponser> {
    let data = data.into_inner();
    let file_name = data.get("file").unwrap().as_str().unwrap();
    // let token = data.get("token").unwrap().as_str().unwrap();
    let root = root.into_inner();
    let file_full_path = root.join(file_name);

    match dir_manage::check(file_full_path).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(mme_obj) => Err(MailManErrResponser::mapping_from_mme(mme_obj)),
    }
}

// POST api/directory/list
pub async fn list(
    data: web::Json<Map<String, Value>>,
    root: web::Data<PathBuf>,
) -> Result<HttpResponse, MailManErrResponser> {
    let data = data.into_inner();
    let file_name = data.get("file").unwrap().as_str().unwrap();
    // let token = data.get("token").unwrap().as_str().unwrap();
    let root = root.into_inner();
    let file_full_path = root.join(file_name);

    match dir_manage::list(file_full_path).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(mme_obj) => Err(MailManErrResponser::mapping_from_mme(mme_obj)),
    }
}
