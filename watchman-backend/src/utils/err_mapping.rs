use actix_web::{
    // error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};

use share_lib::data_structure::MailManErr;

trait MapErrorKey {
    fn response_by_mailman(&mut self) -> HttpResponse;
    fn mapping(&mut self) -> StatusCode;
}

impl<'a> MapErrorKey for MailManErr<'a> {
    fn mapping(&mut self) -> StatusCode {
        match self.code {
            400 => {
                self.key = StatusCode::BAD_REQUEST.as_str();
                StatusCode::BAD_REQUEST
            }
            401 => {
                self.key = StatusCode::UNAUTHORIZED.as_str();
                StatusCode::UNAUTHORIZED
            }
            403 => {
                self.key = StatusCode::FORBIDDEN.as_str();
                StatusCode::FORBIDDEN
            }
            404 => {
                self.key = StatusCode::NOT_FOUND.as_str();
                StatusCode::NOT_FOUND
            }
            501 => {
                self.key = StatusCode::NOT_IMPLEMENTED.as_str();
                StatusCode::NOT_IMPLEMENTED
            }
            503 => {
                self.key = StatusCode::SERVICE_UNAVAILABLE.as_str();
                StatusCode::SERVICE_UNAVAILABLE
            }
            _ => {
                self.key = StatusCode::INTERNAL_SERVER_ERROR.as_str();
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn response_by_mailman(&mut self) -> HttpResponse {
        HttpResponse::build(self.mapping())
            .insert_header(ContentType::json())
            .json(self)
    }
}
