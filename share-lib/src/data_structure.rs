use crate::{log_error, log_info, log_trace, log_warn};
use log::{error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::fmt;

// Self maintance report message - OK
#[derive(Deserialize, Serialize, Debug)]
pub struct MailManOk<'a, T> {
    pub code: u8,
    pub key: &'a str,
    pub data: Option<T>,
}
impl<'a, T> MailManOk<'a, T> {
    pub fn new(code: u8, key: &'a str, data: Option<T>) -> Self {
        log_info!("{}", key);
        MailManOk { code, key, data }
    }
}

// Self maintance report message - Err
#[derive(Deserialize, Serialize, Debug)]
pub struct MailManErr<'a> {
    pub code: u16,
    pub key: &'a str,
    pub msg: String,
    pub level: u8, // 0-2 as warn error fatal
}
impl<'a> MailManErr<'a> {
    pub fn new<E: std::fmt::Display>(code: u16, key: &'a str, error_obj: E, level: u8) -> Self {
        match level {
            0 => log_warn!("{}, {}", key, error_obj),
            1 => log_error!("{}, {}", key, error_obj),
            _ => log_trace!("{}, {}", key, error_obj),
        };
        MailManErr {
            code,
            key,
            msg: error_obj.to_string(),
            level: level,
        }
    }
}

impl fmt::Display for MailManErr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Code: {}, Key: {}, Message: {}",
            self.code, self.key, self.msg
        )
    }
}
