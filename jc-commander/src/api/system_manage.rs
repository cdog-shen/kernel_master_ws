use actix_web::HttpResponse;

use share_lib::{
    data_structure::MailManOk, err_mapping::MailManErrResponser, infrastructure::master_registry,
};

use crate::server;

// send a request to master to refresh the uuid
pub async fn refresh_master() -> Result<HttpResponse, MailManErrResponser> {
    let (base_url, register_name, subsys_uuid) = {
        let config = server::GLOBAL_CONFIG.read().unwrap();
        (
            format!("http://{}:{}/api", config.master_addr, config.master_port),
            config.register_name.clone(),
            config.subsys_uuid.clone(),
        )
    };

    match master_registry::refresh_master_registration(&base_url, &register_name, &subsys_uuid) {
        Ok(res_data) => {
            Ok(HttpResponse::Ok().json(MailManOk::new(200, "Master refreshed", Some(res_data))))
        }
        Err(mme_obj) => Err(MailManErrResponser::mapping_from_mme(mme_obj)),
    }
}
