use std::fmt;

use actix_web::{
    HttpResponse, ResponseError,
    http::{StatusCode, header::ContentType},
};
use serde::Serialize;

use crate::data_structure::MailManErr;

/// a wrapper for MailManErr
///
/// which is need by API(src/api) error response
#[derive(Debug, Serialize)]
pub struct MailManErrResponser {
    pub code: u16,
    pub key: String,
    pub msg: String,
}

impl MailManErrResponser {
    /// get a MailManErr object and map it into MailManErrResponser
    pub fn mapping_from_mme(mme_obj: MailManErr<String>) -> MailManErrResponser {
        let status_key = match mme_obj.code {
            400 => StatusCode::BAD_REQUEST.to_string(),
            401 => StatusCode::UNAUTHORIZED.to_string(),
            403 => StatusCode::FORBIDDEN.to_string(),
            404 => StatusCode::NOT_FOUND.to_string(),
            501 => StatusCode::NOT_IMPLEMENTED.to_string(),
            503 => StatusCode::SERVICE_UNAVAILABLE.to_string(),
            _ => StatusCode::INTERNAL_SERVER_ERROR.to_string(),
        };
        MailManErrResponser {
            code: mme_obj.code,
            key: status_key,
            msg: mme_obj.msg.unwrap_or("No more Msg".to_string()).to_string(),
        }
    }
}

impl fmt::Display for MailManErrResponser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "msg: {}", self.msg)
    }
}

// implement unified error responses for MailManErrResponser
// so that can make MailManErrResponser in error response
impl ResponseError for MailManErrResponser {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(self)
    }

    fn status_code(&self) -> StatusCode {
        match self.code {
            400 => StatusCode::BAD_REQUEST,
            401 => StatusCode::UNAUTHORIZED,
            403 => StatusCode::FORBIDDEN,
            404 => StatusCode::NOT_FOUND,
            501 => StatusCode::NOT_IMPLEMENTED,
            503 => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
