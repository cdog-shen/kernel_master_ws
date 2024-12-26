use actix_web::{web, HttpResponse};

use share_lib::data_structure::MailManOk;

// run cloud api script
pub async fn run(req: web::Path<String>) -> HttpResponse {
    let res: Result<String, String> = Ok(format!("need to impl. data: {}", req.to_string()));
    match res {
        Ok(res_data) => {
            HttpResponse::Ok().json(MailManOk::new(200, "Master refreshed", Some(res_data)))
        }
        Err(err_data) => HttpResponse::InternalServerError().json(err_data.to_string()),
    }
}
