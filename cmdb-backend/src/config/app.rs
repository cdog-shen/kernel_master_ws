use actix_web::web::{self};
use log::info;
use share_lib::log_info;

use crate::api::*;

pub fn config_services(cfg: &mut web::ServiceConfig) {
    log_info!("Configuring routes...");
    cfg.service(
        web::scope("/api")
            .service(
                web::resource("/hey")
                    .route(web::get().to(hey_hi_hello::hey))
                    .route(web::post().to(hey_hi_hello::hey)),
            )
            .service(
                web::resource("/refresh_master")
                    .route(web::post().to(system_manage::refresh_master)),
            )
            .service(
                web::scope("/cmdb")
                    .service(
                        web::resource("/get_all_table")
                            .route(web::get().to(table_manage::get_all_table)),
                    )
                    .service(web::scope("/get").service(
                        web::resource("/{table}").route(web::get().to(table_manage::get_table)),
                    ))
                    .service(web::scope("/new").service(
                        web::resource("/{table}").route(web::post().to(table_manage::new_table)),
                    ))
                    .service(web::scope("/update").service(
                        web::resource("/{table}").route(web::post().to(table_manage::update_table)),
                    ))
                    .service(
                        web::scope("/delete").service(
                            web::resource("/{table}")
                                .route(web::delete().to(table_manage::delete_table)),
                        ),
                    ),
            ),
    );
}

//         .service(
//             web::resource("/{id}")
//                 .route(web::request_type().to(manage::fn)),
