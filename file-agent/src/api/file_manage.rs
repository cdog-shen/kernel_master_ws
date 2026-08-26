use actix_web::{HttpResponse, web};
use serde_json::{Map, Value};
use share_lib::data_structure::MailManErr;
use share_lib::err_mapping::MailManErrResponser;
use std::path::PathBuf;
// use uuid::Uuid;

use crate::service::file_manage;

// POST api/file/check
pub async fn check(
    data: web::Json<Map<String, Value>>,
    root: web::Data<PathBuf>,
) -> Result<HttpResponse, MailManErrResponser> {
    let data = data.into_inner();
    let Some(file_name) = data.get("file").and_then(Value::as_str) else {
        return Err(MailManErrResponser::mapping_from_mme(MailManErr::new(
            400,
            "Bad request",
            Some("missing or invalid param: file".into()),
            1,
        )));
    };
    // let token = data.get("token").unwrap().as_str().unwrap();
    let root = root.into_inner();
    let file_full_path = root.join(file_name);

    match file_manage::check(&file_full_path).await {
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

    match file_manage::download(&file_full_path).await {
        Ok(file) => Ok(HttpResponse::Ok()
            .content_type("application/octet-stream")
            .body(file)),
        Err(mme_obj) => Err(MailManErrResponser::mapping_from_mme(mme_obj)),
    }
}

// POST api/file/delete
pub async fn delete(
    data: web::Json<Map<String, Value>>,
    root: web::Data<PathBuf>,
) -> Result<HttpResponse, MailManErrResponser> {
    let data = data.into_inner();
    let Some(file_name) = data.get("file").and_then(Value::as_str) else {
        return Err(MailManErrResponser::mapping_from_mme(MailManErr::new(
            400,
            "Bad request",
            Some("missing or invalid param: file".into()),
            1,
        )));
    };
    // let token = data.get("token").unwrap().as_str().unwrap();
    let root = root.into_inner();
    let file_full_path = root.join(file_name);

    match file_manage::delete(&file_full_path).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(mme_obj) => Err(MailManErrResponser::mapping_from_mme(mme_obj)),
    }
}

// POST /api/file/upload
pub async fn upload(
    payload: actix_multipart::Multipart,
    root: web::Data<PathBuf>,
) -> Result<HttpResponse, MailManErrResponser> {
    let root = root.into_inner();

    match file_manage::upload(payload, &root).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        // MailManErrResponser 不支持 413，限额超限单独映射以保持原有响应
        Err(mme_obj) if mme_obj.code == 413 => Ok(HttpResponse::PayloadTooLarge()
            .json(serde_json::json!({"code": 413, "msg": "Total size too large"}))),
        Err(mme_obj) => Err(MailManErrResponser::mapping_from_mme(mme_obj)),
    }
}
