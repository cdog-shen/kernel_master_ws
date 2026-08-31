use actix_web::web::{self};
use log::info;
use share_lib::log_info;
use share_lib::middleware::user_auth::{UserAuth, UserAuthConfig};

use crate::api::*;
use crate::config::server::GLOBAL_CONFIG;

// 从全局配置构造统一鉴权中间件（individual 模式下无 master 字段，仅保留 uuid 比对）
fn user_auth() -> UserAuth {
    let config = GLOBAL_CONFIG.read().unwrap();
    UserAuth::new(UserAuthConfig {
        #[cfg(not(feature = "individual"))]
        master_addr: config.master_addr.clone(),
        #[cfg(not(feature = "individual"))]
        master_port: config.master_port,
        subsys_uuid: config.subsys_uuid.clone(),
        authenticate_bypass: config.authenticate_bypass.clone(),
    })
}

pub fn config_services(cfg: &mut web::ServiceConfig) {
    log_info!("Configuring routes...");

    // System management APIs（individual 模式下无 master，主关注册端点编译期裁掉）
    #[cfg(not(feature = "individual"))]
    cfg.service(web::scope("/api/manage").service(
        web::resource("/refresh_master").route(web::post().to(system_manage::refresh_master)),
    ));

    cfg.service(
        web::scope("/api")
            // Health check - public, no auth required
            .service(web::resource("/hey").route(web::get().to(hey_hi_hello::hey)))
            .service(web::resource("/hey").route(web::post().to(hey_hi_hello::hey)))
            // Unified notification APIs
            .service(
                web::scope("/notify")
                    .service(
                        web::resource("/template")
                            .route(web::post().to(notify::send_with_template)),
                    )
                    .wrap(user_auth()),
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
                    .wrap(user_auth()),
            )
            // Notification record APIs
            .service(
                web::scope("/record")
                    .service(web::resource("/get").route(web::get().to(record_manage::get_records)))
                    .wrap(user_auth()),
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
                    .wrap(user_auth()),
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
                    .wrap(user_auth()),
            ),
    );
}
