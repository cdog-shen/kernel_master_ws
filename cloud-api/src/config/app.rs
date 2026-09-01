use actix_web::web::{self};
use log::info;
use share_lib::log_info;

use crate::api::*;

// 路由表人工排版：一行一个 service/resource，rustfmt 跳过本函数以保持布局
#[rustfmt::skip]
pub fn config_services(cfg: &mut web::ServiceConfig) {
    log_info!("Configuring routes...");
    let api_scope = web::scope("/api")
        .service(web::resource("/hey").route(web::post().to(hey_hi_hello::hey)))
        .service(web::scope("/account_db")
            .service(web::resource("/get").route(web::post().to(account_manage::get_all)))
            .service(web::resource("/new").route(web::post().to(account_manage::new)))
            .service(web::resource("/update").route(web::post().to(account_manage::update)))
            .service(web::resource("/delete").route(web::post().to(account_manage::delete))))
        .service(web::scope("/script")
            .service(web::resource("/call").route(web::post().to(script_caller::run)))
            .service(web::resource("/get").route(web::post().to(script_caller::get))));

    // refresh_master 依赖向 master 回源注册，individual 模式下裁掉
    #[cfg(not(feature = "individual"))]
    let api_scope = api_scope
        .service(web::scope("/manage")
            .service(web::resource("/refresh_master").route(web::post().to(system_manage::refresh_master))));

    cfg.service(api_scope);
}

//         .service(
//             web::resource("/{id}")
//                 .route(web::request_type().to(manage::fn)),
