use actix_web::web;
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
            .service(web::resource("/reload").route(web::post().to(system_manage::reload_config)))
            .service(
                web::scope("/auth")
                    .service(web::resource("/login").route(web::post().to(account_manage::login)))
                    .service(web::resource("/me/{id}").route(web::get().to(account_manage::get_me)))
                    .service(web::resource("/logout").route(web::post().to(account_manage::logout)))
            )
            .service(
                web::resource("/user")
                    .route(web::get().to(account_manage::get_all))
                    .route(web::post().to(account_manage::signup))
                    .route(web::patch().to(account_manage::user_update))
            )
            .service(
                web::resource("/group")
                    .route(web::get().to(group_manage::all_group))
                    .route(web::post().to(group_manage::new_group))
                    .route(web::patch().to(group_manage::update_group))
                    .route(web::delete().to(group_manage::delete_group))
            )
            .service(
                web::resource("/service")
                    .route(web::get().to(service_manage::all_service))
                    .route(web::post().to(service_manage::new_service))
                    .route(web::post().to(service_manage::update_service))
                    .route(web::delete().to(service_manage::delete_service))
            )
            .service(
                web::resource("/access")
                .route(web::get().to(access_manage::all_access))
                .route(web::post().to(access_manage::new_access))
                .route(web::patch().to(access_manage::update_access))
                .route(web::delete().to(access_manage::delete_access))
            )
            .service(
                web::resource("/subsystem")
                .route(web::get().to(subsys_manage::all_subsys))
                .route(web::post().to(subsys_manage::new_subsys))
                .route(web::patch().to(subsys_manage::update_subsys))
                .route(web::delete().to(subsys_manage::delete_subsys))
            )
            .service(
                web::scope("/subsystem_call").service(
                    web::resource("/{subsystem_name}")
                        .route(web::post().to(subsys_manage::call_subsys)),
                ),
            ),
    );
}

//         .service(
//             web::resource("/{id}")
//                 .route(web::request_type().to(manage::fn)),
