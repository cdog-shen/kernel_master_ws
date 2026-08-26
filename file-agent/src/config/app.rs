use actix_web::web::{self};
use log::info;
use share_lib::log_info;

use crate::api::*;

pub fn config_services(cfg: &mut web::ServiceConfig) {
    log_info!("Configuring routes...");
    cfg.service(
        web::scope("/api")
            .service(web::resource("/hey").route(web::post().to(hey_hi_hello::hey)))
            .service(
                web::scope("/manage").service(
                    web::resource("/refresh_master")
                        .route(web::post().to(system_manage::refresh_master)),
                ),
            )
            .service(
                web::scope("/token")
                    .service(web::resource("/gen").route(web::post().to(token::generate))), // .service(web::resource("/delete").route(web::post().to(table_manage::delete_table)))
            )
            .service(
                web::scope("/file")
                    .service(web::resource("/check").route(web::post().to(file_manage::check)))
                    .service(
                        web::resource("/download/{path:.*}")
                            .route(web::get().to(file_manage::download)),
                    )
                    .service(web::resource("/upload").route(web::post().to(file_manage::upload)))
                    .service(web::resource("/delete").route(web::post().to(file_manage::delete))),
            )
            .service(
                web::scope("/directory")
                    .service(web::resource("/check").route(web::post().to(dir_manage::check)))
                    .service(web::resource("/list").route(web::post().to(dir_manage::list))), // .service(web::resource("/download").route(web::post().to(file_manage::update_table)))
                                                                                              // .service(web::resource("/delete").route(web::post().to(table_manage::delete_table)))
            ),
    );
}

//         .service(
//             web::resource("/{id}")
//                 .route(web::request_type().to(manage::fn)),
