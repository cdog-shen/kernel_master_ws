use actix_web::Result;
use chrono::{DateTime, Local};
use serde_json::{Value, json};
use share_lib::data_structure::{MailManErr, MailManOk};
use std::path::PathBuf;

use crate::infra::file_op;

pub async fn check<'a>(
    path: PathBuf,
) -> Result<MailManOk<'a, serde_json::Value>, MailManErr<'a, String>> {
    if file_op::is_file(&path) {
        Ok(MailManOk::new(
            200,
            "Service: File check",
            Some(json!({"type": "normal"})),
        ))
    } else if file_op::is_dir(&path) {
        Ok(MailManOk::new(
            200,
            "Service: File check",
            Some(json!({"type": "directory"})),
        ))
    } else {
        Err(MailManErr::new(
            404,
            "Service: File check",
            Some("No item".to_string()),
            1,
        ))
    }
}

pub async fn list<'a>(
    path: PathBuf,
) -> Result<MailManOk<'a, serde_json::Value>, MailManErr<'a, String>> {
    if file_op::is_file(&path) {
        return Err(MailManErr::new(
            200,
            "Service: File check",
            Some("It's a File".to_string()),
            1,
        ));
    } else if file_op::is_dir(&path) {
        MailManOk::new(
            200,
            "Service: File check",
            Some(json!({"type": "directory"})),
        );
    } else {
        return Err(MailManErr::new(
            404,
            "Service: File check",
            Some("No item".to_string()),
            1,
        ));
    };

    let entries = file_op::read_dir(&path)
        .map_err(|e| MailManErr::new(500, "Service: read_dir failed", Some(e.to_string()), 1))?;

    let mut items = Vec::new();
    for entry in entries {
        let file_type = if entry.is_dir {
            "directory"
        } else if entry.is_file {
            "file"
        } else {
            "other"
        };

        // 取修改时间 → chrono → RFC 3339 字符串
        let update_time: DateTime<Local> = DateTime::from(entry.modified);

        items.push(json!({
            "name": entry.name,
            "type": file_type,
            "update_time": update_time.to_rfc3339()
        }));
    }

    Ok(MailManOk::new(
        200,
        "Service: list success",
        Some(Value::Array(items)),
    ))
}
