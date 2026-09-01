use actix_web::web;
use log::info;
use share_lib::log_info;

use crate::api::*;
use crate::middleware::auth_middleware::{JwtAuth, PermissionCheck};

// 路由表人工排版：一行一个 service/resource，rustfmt 跳过本函数以保持布局
#[rustfmt::skip]
pub fn config_services(cfg: &mut web::ServiceConfig) {
    log_info!("Configuring routes...");

    // 子系统回源鉴权端点（POST /api/auth/verify）
    // 必须注册在 /api scope 之前：actix 路由按注册顺序命中，/api scope 前缀命中后
    // 内部失配不会回退到同级条目；且该端点自带子系统 uuid 调用方认证，
    // 不能经过 /api scope 上挂的 JwtAuth / PermissionCheck 中间件。
    cfg.service(web::resource("/api/auth/verify").route(web::post().to(auth_manage::verify)));

    cfg.service(
        // API scope
        web::scope("/api")
            // healthy check
            .service(web::resource("/hey")
                .route(web::get().to(hey_hi_hello::hey))
                .route(web::post().to(hey_hi_hello::hey)))
            .wrap(JwtAuth)
            .wrap(PermissionCheck)
            // Hot reload
            .service(web::resource("/reload").route(web::post().to(system_manage::reload_config)))
            // Auth
            .service(web::scope("/auth")
                .service(web::resource("/login").route(web::post().to(account_manage::login)))
                .service(web::resource("/logout").route(web::post().to(account_manage::logout)))
                // UID check (No need to check permission)
                .service(web::resource("/me/{id}").route(web::get().to(account_manage::get_me)))
                .wrap(JwtAuth))
            // user management
            .service(web::resource("/user")
                .route(web::get().to(account_manage::all_user))
                .route(web::post().to(account_manage::signup))
                .route(web::patch().to(account_manage::user_update)))
            .wrap(JwtAuth)
            .wrap(PermissionCheck)
            // group (role) management
            .service(web::resource("/group")
                .route(web::get().to(group_manage::all_group))
                .route(web::post().to(group_manage::new_group))
                .route(web::patch().to(group_manage::update_group))
                .route(web::delete().to(group_manage::delete_group)))
            .wrap(JwtAuth)
            .wrap(PermissionCheck)
            // service management
            .service(web::resource("/service")
                .route(web::get().to(service_manage::all_service))
                .route(web::post().to(service_manage::new_service))
                .route(web::patch().to(service_manage::update_service))
                .route(web::delete().to(service_manage::delete_service)))
            .wrap(JwtAuth)
            .wrap(PermissionCheck)
            // access management
            .service(web::resource("/access")
                .route(web::get().to(access_manage::all_access))
                .route(web::post().to(access_manage::new_access))
                .route(web::patch().to(access_manage::update_access))
                .route(web::delete().to(access_manage::delete_access)))
            .wrap(JwtAuth)
            .wrap(PermissionCheck)
            // subsystem management
            .service(web::resource("/subsystem")
                .route(web::get().to(subsys_manage::all_subsys))
                .route(web::post().to(subsys_manage::new_subsys))
                .route(web::patch().to(subsys_manage::update_subsys))
                .route(web::delete().to(subsys_manage::delete_subsys)))
            .wrap(JwtAuth)
            .wrap(PermissionCheck)
            // Subsystem JSON RPC call
            .service(web::scope("/subsystem_call")
                .service(web::resource("/{subsystem_name}").route(web::post().to(subsys_manage::call_subsys))))
            .wrap(JwtAuth)
            .wrap(PermissionCheck)
            // webhook management
            .service(web::resource("/webhook")
                .route(web::get().to(webhook_manage::all_webhook))
                .route(web::post().to(webhook_manage::new_webhook))
                .route(web::patch().to(webhook_manage::update_webhook))
                .route(web::delete().to(webhook_manage::delete_webhook)))
            .wrap(JwtAuth)
            .wrap(PermissionCheck)
            // webhook call
            .service(web::scope("/webhook")
                .service(web::resource("/{token}")
                    .route(web::post().to(webhook_manage::post_webhook))
                    .route(web::get().to(webhook_manage::get_webhook)))),
    );
}

//         .service(
//             web::resource("/{id}")
//                 .route(web::request_type().to(manage::fn)),
