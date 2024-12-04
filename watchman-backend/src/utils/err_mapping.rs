use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};
use serde::{Deserialize, Serialize};
use std::fmt;

use share_lib::data_structure::MailManErr;

#[derive(Debug,Serialize)]
pub struct MailManErrResponser {
    pub mme_msg: String,
}

pub trait MapErrorKey {
    // fn response_by_mailman(&mut self) -> HttpResponse;
    fn mapping_to_status(&self) -> StatusCode;
}

// impl<'a> MailManErrResponser<'a> {
//     pub fn new(status_code: StatusCode,warp_obj: MailManErr,) {
//         MailManErrResponser {}
//     }
// }

impl<'a> MapErrorKey for MailManErr<'a> {
    fn mapping_to_status(&self) -> StatusCode {
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

impl fmt::Display for MailManErrResponser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "msg: {}", self.mme_msg)
    }
}

impl ResponseError for MailManErrResponser {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(self)
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}