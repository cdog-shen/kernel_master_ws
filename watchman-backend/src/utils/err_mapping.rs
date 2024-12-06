use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};
use serde::Serialize;
use std::fmt;

use share_lib::data_structure::MailManErr;

#[derive(Debug, Serialize)]
pub struct MailManErrResponser {
    pub code: u16,
    pub key: String,
    pub msg: String,
}

pub trait MapErrorKey {
    // fn response_by_mailman(&mut self) -> HttpResponse;
    fn mapping_from_mme(mme_obj: MailManErr) -> MailManErrResponser;
}

impl<'a> MapErrorKey for MailManErrResponser {
    fn mapping_from_mme(mme_obj: MailManErr) -> MailManErrResponser {
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
            msg: mme_obj.msg,
        }
    }
}

impl<'a> fmt::Display for MailManErrResponser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "msg: {}", self.msg)
    }
}

impl<'a> ResponseError for MailManErrResponser {
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
