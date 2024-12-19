use actix_web::HttpResponse;

use share_lib::data_structure::MailManOk;

use crate::{server, utils::err_mapping::MailManErrResponser};

// POST api/reload
pub async fn reload_config() -> Result<HttpResponse, MailManErrResponser> {
    match server::GLOBAL_CONFIG.write().unwrap().reload() {
        Ok(_) => Ok(HttpResponse::Ok().json(MailManOk::new(200, "config load DONE", None::<&str>))),
        Err(err_mm) => Err(MailManErrResponser::mapping_from_mme(err_mm)),
    }
}
