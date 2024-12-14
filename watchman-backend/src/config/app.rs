use actix_web::web;
use log::info;
use share_lib::log_info;

use crate::api::*;

pub fn config_services(cfg: &mut web::ServiceConfig) {
    log_info!("Configuring routes...");
    cfg.service(
        web::scope("/api")
            .service(web::resource("/hey")
                .route(web::get().to(hey_hi_hello::hey))
                .route(web::post().to(hey_hi_hello::hey))
            )
            .service(web::scope("/auth")
                .service(web::resource("/all_user").route(web::get().to(account_manage::get_all)))
                .service(web::resource("/me/{id}").route(web::get().to(account_manage::get_me)))
                .service(web::resource("/signup").route(web::post().to(account_manage::signup)))
                .service(web::resource("/login").route(web::post().to(account_manage::login)))
                .service(web::resource("/logout").route(web::post().to(account_manage::logout)))
                .service(web::resource("/user_update").route(web::post().to(account_manage::user_update)))
            )
            .service(web::scope("/group_control")
                .service(web::resource("/all_group").route(web::get().to(group_manage::all_group)))
                .service(web::resource("/new_group").route(web::post().to(group_manage::new_group)))
                .service(web::resource("/update_group").route(web::post().to(group_manage::update_group)))
                .service(web::resource("/delete_group").route(web::delete().to(group_manage::delete_group)))
            )
            .service(web::scope("/service_control")
                .service(web::resource("/all_service").route(web::get().to(service_manage::all_service)))
                .service(web::resource("/new_service").route(web::post().to(service_manage::new_service)))
                .service(web::resource("/update_service").route(web::post().to(service_manage::update_service)))
                .service(web::resource("/delete_service").route(web::delete().to(service_manage::delete_service)))
            )
            .service(web::scope("/access_control")
                .service(web::resource("/all_access").route(web::get().to(access_manage::all_access)))
                .service(web::resource("/new_access").route(web::post().to(access_manage::new_access)))
                .service(web::resource("/update_access").route(web::post().to(access_manage::update_access)))
                .service(web::resource("/delete_access").route(web::delete().to(access_manage::delete_access)))
            )
            .service(web::scope("/subsystem_control")
                .service(web::resource("/all_subsystem").route(web::get().to(subsys_manage::all_subsys)))
                .service(web::resource("/new_subsystem").route(web::post().to(subsys_manage::new_subsys)))
                .service(web::resource("/update_subsystem").route(web::post().to(subsys_manage::update_subsys)))
                .service(web::resource("/delete_subsystem").route(web::delete().to(subsys_manage::delete_subsys)))
            )

        );
}


//         .service(
//             web::resource("/{id}")
//                 .route(web::request_type().to(manage::fn)),