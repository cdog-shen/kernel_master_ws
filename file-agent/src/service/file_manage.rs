use actix_web::Result;
use serde_json::json;
use share_lib::data_structure::{MailManErr, MailManOk};
use std::{fs, path::PathBuf};

pub fn check<'a>(
    path: &PathBuf,
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
            401,
            "Service: File check",
            Some("No item".to_string()),
            1,
        ))
    }
}

pub fn download<'a>(path: &PathBuf) -> Result<Vec<u8>, MailManErr<'a, String>> {
    if path.is_file() {
        MailManOk::new(200, "Service: File check", Some(json!({"type": "normal"})));
    } else {
        return Err(MailManErr::new(
            401,
            "Service: File check",
            Some("No item".to_string()),
            1,
        ));
    }

    match fs::read(path) {
        Ok(file_bytes) => Ok(file_bytes),
        Err(e) => Err(MailManErr::new(
            500,
            "Service: File download",
            Some(e.to_string()),
            1,
        )),
    }
}

pub fn delete<'a>(path: &PathBuf) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    if path.is_file() {
        MailManOk::new(200, "Service: File check", Some(json!({"type": "normal"})));
    } else {
        return Err(MailManErr::new(
            401,
            "Service: File delete",
            Some("No item".to_string()),
            1,
        ));
    }

    match fs::remove_file(path) {
        Ok(_) => Ok(MailManOk::new(
            200,
            "Service: File delete",
            Some(format!("{} deleted", path.clone().to_str().unwrap())),
        )),
        Err(e) => Err(MailManErr::new(
            500,
            "Service: File delete",
            Some(e.to_string()),
            1,
        )),
    }
}
