use actix_web::web::{self};
use log::info;
use share_lib::log_info;

use crate::api::*;
use crate::middleware::auth_middleware::Authentication;

pub fn config_services(cfg: &mut web::ServiceConfig) {
    log_info!("Configuring routes...");
    cfg.service(
        web::scope("/api")
            // Health check - public, no auth required
            .service(web::resource("/hey").route(web::post().to(hey_hi_hello::hey)))
            // Unified notification APIs
            .service(
                web::scope("/notify")
                    .service(
                        web::resource("/template")
                            .route(web::post().to(notify::send_with_template)),
                    )
                    .wrap(Authentication),
            )
            // Template management APIs
            .service(
                web::scope("/template")
                    .service(web::resource("/get").route(web::get().to(notify::get_templates)))
                    .service(web::resource("/new").route(web::post().to(notify::create_template)))
                    .service(
                        web::resource("/update").route(web::post().to(notify::update_template)),
                    )
                    .service(
                        web::resource("/delete").route(web::post().to(notify::delete_template)),
                    )
                    .wrap(Authentication),
            )
            // Notification record APIs
            .service(
                web::scope("/record")
                    .service(web::resource("/get").route(web::get().to(notify::get_records)))
                    .wrap(Authentication),
            )
            // Channel config APIs
            .service(
                web::scope("/channel")
                    .service(
                        web::resource("/get").route(web::get().to(notify::get_channel_configs)),
                    )
                    .service(
                        web::resource("/update")
                            .route(web::post().to(notify::update_channel_config)),
                    )
                    .wrap(Authentication),
            )
            // Alias management APIs
            .service(
                web::scope("/alias")
                    .service(web::resource("/get").route(web::get().to(notify::get_aliases)))
                    .service(web::resource("/new").route(web::post().to(notify::create_alias)))
                    .service(web::resource("/update").route(web::post().to(notify::update_alias)))
                    .service(web::resource("/delete").route(web::post().to(notify::delete_alias)))
                    .wrap(Authentication),
            ),
    );
}
