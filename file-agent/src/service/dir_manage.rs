use actix_web::Result;
use chrono::{DateTime, Local};
use serde_json::{Value, json};
use share_lib::data_structure::{MailManErr, MailManOk};
use std::{fs, path::PathBuf};

pub fn check<'a>(
    path: PathBuf,
) -> Result<MailManOk<'a, serde_json::Value>, MailManErr<'a, String>> {
    if path.is_file() {
        Ok(MailManOk::new(
            200,
            "Service: File check",
            Some(json!({"type": "normal"})),
        ))
    } else if path.is_dir() {
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

pub fn list<'a>(path: PathBuf) -> Result<MailManOk<'a, serde_json::Value>, MailManErr<'a, String>> {
    if path.is_file() {
        return Err(MailManErr::new(
            200,
            "Service: File check",
            Some("It's a File".to_string()),
            1,
        ));
    } else if path.is_dir() {
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

    let mut items = Vec::new();
    for entry in fs::read_dir(&path)
        .map_err(|e| MailManErr::new(500, "Service: read_dir failed", Some(e.to_string()), 1))?
    {
        let entry = entry.map_err(|e| {
            MailManErr::new(500, "Service: DirEntry failed", Some(e.to_string()), 1)
        })?;

        let name = entry
            .file_name()
            .into_string()
            .unwrap_or_else(|_| String::from("<invalid utf8>"));

        let md = entry.metadata().map_err(|e| {
            MailManErr::new(500, "Service: metadata failed", Some(e.to_string()), 1)
        })?;

        let file_type = if md.is_dir() {
            "directory"
        } else if md.is_file() {
            "file"
        } else {
            "other"
        };

        // 取修改时间 → chrono → RFC 3339 字符串
        let update_time: DateTime<Local> = DateTime::from(md.modified().map_err(|e| {
            MailManErr::new(500, "Service: modified failed", Some(e.to_string()), 1)
        })?);

        items.push(json!({
            "name": name,
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
