use actix_multipart::Multipart;
use actix_web::Result;
use futures::{StreamExt, TryStreamExt};
use sanitize_filename::sanitize;
use serde_json::json;
use share_lib::data_structure::{MailManErr, MailManOk};
use std::path::{Path, PathBuf};

use crate::config::server::GLOBAL_CONFIG;
use crate::infra::file_op;

pub async fn check<'a>(
    path: &PathBuf,
) -> Result<MailManOk<'a, serde_json::Value>, MailManErr<'a, String>> {
    if file_op::is_file(path) {
        Ok(MailManOk::new(
            200,
            "Service: File check",
            Some(json!({"type": "normal"})),
        ))
    } else if file_op::is_dir(path) {
        Ok(MailManOk::new(
            200,
            "Service: File check",
            Some(json!({"type": "directory"})),
        ))
    } else {
        Err(MailManErr::new(
            401,
            "Service: File check",
            Some("No item".to_string()),
            1,
        ))
    }
}

pub async fn download<'a>(path: &Path) -> Result<Vec<u8>, MailManErr<'a, String>> {
    if file_op::is_file(path) {
        MailManOk::new(200, "Service: File check", Some(json!({"type": "normal"})));
    } else {
        return Err(MailManErr::new(
            401,
            "Service: File check",
            Some("No item".to_string()),
            1,
        ));
    }

    match file_op::read_file(path) {
        Ok(file_bytes) => Ok(file_bytes),
        Err(e) => Err(MailManErr::new(
            500,
            "Service: File download",
            Some(e.to_string()),
            1,
        )),
    }
}

pub async fn prepare_path(original: &str, root: &PathBuf) -> Result<PathBuf, String> {
    let safe = sanitize(original);
    let dir = root.join("upload");
    file_op::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(safe))
}

/// upload 编排：multipart 流式解析 → 限额累计 → 原子写入
pub async fn upload<'a>(
    mut payload: Multipart,
    root: &PathBuf,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    let limit = GLOBAL_CONFIG.read().unwrap().file_size_limit;

    let mut total: u64 = 0;

    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(|e| MailManErr::new(500, "Multipart error", Some(e.to_string()), 1))?
    {
        let cd = field.content_disposition();
        let Some(filename) = cd.as_ref().and_then(|cd| cd.get_filename()) else {
            continue;
        };

        let path = prepare_path(filename, root)
            .await
            .map_err(|e| MailManErr::new(500, "Prepare path", Some(e), 1))?;

        let mut file = file_op::create_file(&path)
            .await
            .map_err(|e| MailManErr::new(500, "Create file", Some(e.to_string()), 1))?;

        while let Some(chunk) = field.next().await {
            let chunk =
                chunk.map_err(|e| MailManErr::new(500, "Read chunk", Some(e.to_string()), 1))?;

            total += chunk.len() as u64;

            if limit > 0 && total > limit {
                return Err(MailManErr::new(
                    413,
                    "Upload limit",
                    Some("Total size too large".to_string()),
                    1,
                ));
            }

            file_op::write_chunk(&mut file, &chunk)
                .await
                .map_err(|e| MailManErr::new(500, "Write file", Some(e.to_string()), 1))?;
        }
    }

    if total == 0 {
        return Err(MailManErr::new(
            400,
            "No file",
            Some("no file uploaded".to_string()),
            1,
        ));
    }

    let total_mb = total / 1024 / 1024;

    Ok(MailManOk::new(
        200,
        "File Upload success",
        Some(format!("uploaded {total} bytes, about {total_mb}M")),
    ))
}

pub async fn delete<'a>(path: &Path) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    if file_op::is_file(path) {
        MailManOk::new(200, "Service: File check", Some(json!({"type": "normal"})));
    } else {
        return Err(MailManErr::new(
            401,
            "Service: File delete",
            Some("No item".to_string()),
            1,
        ));
    }

    match file_op::remove_file(path) {
        Ok(_) => Ok(MailManOk::new(
            200,
            "Service: File delete",
            Some(format!("{} deleted", path.to_str().unwrap())),
        )),
        Err(e) => Err(MailManErr::new(
            500,
            "Service: File delete",
            Some(e.to_string()),
            1,
        )),
    }
}
