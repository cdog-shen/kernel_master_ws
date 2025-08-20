use chrono::{DateTime, Local, TimeDelta};
use std::path::PathBuf;
use uuid::Uuid;

enum FileOperation {
    Upload,
    Download,
}

impl FileOperation {
    fn from_value(op: serde_json::Value) -> Result<Self, String> {
        match op {
            serde_json::Value::String(val) if val == "upload" => {
                Ok(FileOperation::Upload)
            }
            serde_json::Value::String(val) if val == "download" => {
                Ok(FileOperation::Download)
            }
            _ => Err("Wrong FileOperation".to_string()),
        }
    }

    fn from_string(op: String) -> Result<Self, String> {
        match op {
            val if val == "upload" => Ok(FileOperation::Upload),
            val if val == "download" => Ok(FileOperation::Download),
            _ => Err("Wrong FileOperation".to_string()),
        }
    }
}

pub struct FileOpInfo {
    path: PathBuf,
    token: Uuid,
    operation: FileOperation,
    expire: DateTime<Local>,
}

impl FileOpInfo {
    pub fn new(
        path: PathBuf,
        token: Uuid,
        operation: FileOperation,
        lasting: Option<TimeDelta>,
    ) -> Self {
        FileOpInfo {
            path,
            token,
            operation,
            expire: Local::now() + lasting.unwrap_or(TimeDelta::seconds(600)),
        }
    }
}
