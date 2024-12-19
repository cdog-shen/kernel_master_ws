use actix_web::web;
use log::info;
use share_lib::log_info;

use crate::api::*;

pub fn config_services(cfg: &mut web::ServiceConfig) {
    log_info!("Configuring routes...");
    cfg.service(
        web::scope("/api").service(
            web::resource("/hey")
                .route(web::get().to(hey_hi_hello::hey))
                .route(web::post().to(hey_hi_hello::hey)),
        ),
        // .service(web::resource("/reload").route(web::post().to(system_manage::reload_config))),
    );
}

//         .service(
//             web::resource("/{id}")
//                 .route(web::request_type().to(manage::fn)),
