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
            .service(
                web::scope("/auth")
                    .service(web::resource("/signup").route(web::post().to(account_manage::signup)))
                    .service(web::resource("/login").route(web::post().to(account_manage::login)))
                    .service(web::resource("/logout").route(web::post().to(account_manage::logout)))
                    .service(web::resource("/user_update").route(web::post().to(account_manage::user_update))) 
                    //         .service(
                    //             web::resource("/{id}")
                    //                 .route(web::get().to(address_book_controller::find_by_id))
                    //                 .route(web::put().to(address_book_controller::update))
                    //                 .route(web::delete().to(address_book_controller::delete)),
            )
            .service(web::scope("/access_control")
                .service(web::resource("/all_group").route(web::get().to(group_manage::all_group)))
                .service(web::resource("/new_group").route(web::post().to(group_manage::new_group)))
                .service(web::resource("/update_group").route(web::post().to(group_manage::update_group)))
                .service(web::resource("/delete_group").route(web::delete().to(group_manage::delete_group)))

            ),
    );
}
