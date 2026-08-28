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
            .service(web::resource("/hey").route(web::get().to(hey_hi_hello::hey)))
            .service(web::resource("/hey").route(web::post().to(hey_hi_hello::hey)))
            // System management APIs
            .service(
                web::scope("/manage").service(
                    web::resource("/refresh_master")
                        .route(web::post().to(system_manage::refresh_master)),
                ),
            )
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
                    .service(
                        web::resource("/get").route(web::get().to(template_manage::get_templates)),
                    )
                    .service(
                        web::resource("/help")
                            .route(web::get().to(template_manage::get_template_help))
                            .route(web::post().to(template_manage::post_template_help)),
                    )
                    .service(
                        web::resource("/new")
                            .route(web::post().to(template_manage::create_template)),
                    )
                    .service(
                        web::resource("/update")
                            .route(web::post().to(template_manage::update_template)),
                    )
                    .service(
                        web::resource("/delete")
                            .route(web::post().to(template_manage::delete_template)),
                    )
                    .wrap(Authentication),
            )
            // Notification record APIs
            .service(
                web::scope("/record")
                    .service(web::resource("/get").route(web::get().to(record_manage::get_records)))
                    .wrap(Authentication),
            )
            // Channel config APIs
            .service(
                web::scope("/channel")
                    .service(
                        web::resource("/get")
                            .route(web::get().to(channel_manage::get_channel_configs)),
                    )
                    .service(
                        web::resource("/update")
                            .route(web::post().to(channel_manage::update_channel_config)),
                    )
                    .wrap(Authentication),
            )
            // Alias management APIs
            .service(
                web::scope("/alias")
                    .service(web::resource("/get").route(web::get().to(alias_manage::get_aliases)))
                    .service(
                        web::resource("/new").route(web::post().to(alias_manage::create_alias)),
                    )
                    .service(
                        web::resource("/update").route(web::post().to(alias_manage::update_alias)),
                    )
                    .service(
                        web::resource("/delete").route(web::post().to(alias_manage::delete_alias)),
                    )
                    .wrap(Authentication),
            ),
    );
}
