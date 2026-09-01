use actix_web::web::{self};
use log::info;
use share_lib::log_info;

use crate::api::*;

// 路由表人工排版：一行一个 service/resource，rustfmt 跳过本函数以保持布局
#[rustfmt::skip]
pub fn config_services(cfg: &mut web::ServiceConfig) {
    log_info!("Configuring routes...");
    let api_scope = web::scope("/api")
        .service(web::resource("/hey").route(web::post().to(hey_hi_hello::hey)));
    // individual 独立运行模式无 master 可刷新，裁掉 refresh_master 路由
    #[cfg(not(feature = "individual"))]
    let api_scope = api_scope
        .service(web::scope("/manage")
            .service(web::resource("/refresh_master").route(web::post().to(system_manage::refresh_master))));
    cfg.service(
        api_scope
            .service(web::scope("/log")
                .service(web::resource("/get").route(web::post().to(job_log::get_all)))
                // .service(web::resource("/new").route(web::post().to(job_log::new)))
                .service(web::resource("/update").route(web::post().to(job_log::update)))
                .service(web::resource("/delete").route(web::post().to(job_log::delete))))
            .service(web::scope("/cron")
                .service(web::resource("/get").route(web::post().to(cron_job::get_all)))
                .service(web::resource("/new").route(web::post().to(cron_job::new)))
                .service(web::resource("/update").route(web::post().to(cron_job::update)))
                .service(web::resource("/delete").route(web::post().to(cron_job::delete)))
                .service(web::resource("/refresh").route(web::post().to(cron_job::refresh))))
            .service(web::scope("/script")
                .service(web::resource("/sync").route(web::post().to(script_caller::call_sync)))
                .service(web::resource("/async").route(web::post().to(script_caller::call_async)))),
    );
}

//         .service(
//             web::resource("/{id}")
//                 .route(web::request_type().to(manage::fn)),
