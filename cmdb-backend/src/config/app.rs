use actix_web::web::{self};
use log::info;
use share_lib::log_info;

use crate::api::*;

// 路由表人工排版：一行一个 service/resource，rustfmt 跳过本函数以保持布局
#[rustfmt::skip]
pub fn config_services(cfg: &mut web::ServiceConfig) {
    log_info!("Configuring routes...");
    cfg.service(
        web::scope("/api")
            .service(web::resource("/hey").route(web::post().to(hey_hi_hello::hey)))
            .service(web::scope("/table")
                .service(web::resource("/query").route(web::post().to(table_manage::query_table)))
                .service(web::resource("/new").route(web::post().to(table_manage::new_table)))
                .service(web::resource("/update").route(web::post().to(table_manage::update_table)))
                .service(web::resource("/delete").route(web::post().to(table_manage::delete_table)))),
    );

    // 与 watchman 交互的管理路由，individual 独立运行模式下裁掉
    #[cfg(not(feature = "individual"))]
    cfg.service(web::scope("/api/manage")
        .service(web::resource("/refresh_master").route(web::post().to(system_manage::refresh_master)))
        .service(web::resource("/register_help").route(web::get().to(system_manage::register_help))));
}

//         .service(
//             web::resource("/{id}")
//                 .route(web::request_type().to(manage::fn)),
