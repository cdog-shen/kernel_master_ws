use actix_web::{HttpResponse, web};
use futures::StreamExt;
use futures::TryStreamExt;
use serde_json::{Map, Value};
use share_lib::data_structure::MailManErr;
use share_lib::data_structure::MailManOk;
use share_lib::err_mapping::MailManErrResponser;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
// use uuid::Uuid;

use crate::{server::GLOBAL_CONFIG, service::file_manage};

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
    let file_name = data.get("file").unwrap().as_str().unwrap();
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
    mut payload: actix_multipart::Multipart,
    root: web::Data<PathBuf>,
) -> Result<HttpResponse, MailManErrResponser> {
    let root = root.into_inner();
    let limit = GLOBAL_CONFIG.read().unwrap().file_size_limit;

    let mut total: u64 = 0;

    while let Some(mut field) = payload.try_next().await.map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr::new(
            500,
            "Multipart error",
            Some(e.to_string()),
            1,
        ))
    })? {
        let cd = field.content_disposition();
        let Some(filename) = cd.as_ref().and_then(|cd| cd.get_filename()) else {
            continue;
        };

        let path = file_manage::prepare_path(filename, &root)
            .await
            .map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(
                    500,
                    "Prepare path",
                    Some(e),
                    1,
                ))
            })?;

        let mut file = File::create(&path).await.map_err(|e| {
            MailManErrResponser::mapping_from_mme(MailManErr::new(
                500,
                "Create file",
                Some(e.to_string()),
                1,
            ))
        })?;

        while let Some(chunk) = field.next().await {
            let chunk = chunk.map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(
                    500,
                    "Read chunk",
                    Some(e.to_string()),
                    1,
                ))
            })?;

            total += chunk.len() as u64;

            if limit > 0 && total > limit {
                return Ok(HttpResponse::PayloadTooLarge()
                    .json(serde_json::json!({"code": 413, "msg": "Total size too large"})));
            }
            file.write_all(&chunk).await.map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(
                    500,
                    "Write file",
                    Some(e.to_string()),
                    1,
                ))
            })?;
        }
    }

    if total == 0 {
        return Err(MailManErrResponser::mapping_from_mme(MailManErr::new(
            400,
            "No file",
            Some("no file uploaded".into()),
            1,
        )));
    }

    let total_mb = total / 1024 / 1024;

    Ok(HttpResponse::Ok().json(MailManOk::new(
        200,
        "File Upload success",
        Some(format!("uploaded {total} bytes, about {total_mb}M")),
    )))
}
