use actix_web::{HttpResponse, web};
use serde_json::{Map, Value};
use std::path::PathBuf;
// use uuid::Uuid;

use crate::{service::file_manage, util::err_mapping::MailManErrResponser};

// POST api/file/check
pub async fn check(
    data: web::Json<Map<String, Value>>,
    root: web::Data<PathBuf>,
) -> Result<HttpResponse, MailManErrResponser> {
    let data = data.into_inner();
    let file_name = data.get("file").unwrap().as_str().unwrap();
    // let token = data.get("token").unwrap().as_str().unwrap();
    let root = root.into_inner();
    let file_full_path = root.join(file_name);

    match file_manage::check(&file_full_path) {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(mme_obj) => Err(MailManErrResponser::mapping_from_mme(mme_obj)),
    }
}

// POST api/file/download
pub async fn download(
    data: web::Path<String>,
    root: web::Data<PathBuf>,
) -> Result<HttpResponse, MailManErrResponser> {
    let fp: String = data.to_string();
    // let token = data.get("token").unwrap().as_str().unwrap();
    let root = root.into_inner();
    let file_full_path = root.join(fp);

    match file_manage::download(&file_full_path) {
        Ok(file) => Ok(HttpResponse::Ok()
            .content_type("application/octet-stream")
            .body(file)),
        Err(mme_obj) => Err(MailManErrResponser::mapping_from_mme(mme_obj)),
    }
}

// POST api/file/delete
pub async fn delete(
    data: web::Path<String>,
    root: web::Data<PathBuf>,
) -> Result<HttpResponse, MailManErrResponser> {
    let fp: String = data.to_string();
    // let token = data.get("token").unwrap().as_str().unwrap();
    let root = root.into_inner();
    let file_full_path = root.join(fp);

    match file_manage::delete(&file_full_path) {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(mme_obj) => Err(MailManErrResponser::mapping_from_mme(mme_obj)),
    }
}
