// std import
use log::info;
use std::path::PathBuf;
// rt import
use actix_cors::Cors;
use actix_web::dev::Service;
use actix_web::web;
use actix_web::{App, HttpServer, http};
use futures::FutureExt;
// db utils import
// use diesel::PgConnection;
// use diesel::r2d2::ConnectionManager;
// db models

// share-lib import
use share_lib::data_structure::MailManOk;
use share_lib::middleware::user_auth::{UserAuth, UserAuthConfig};
use share_lib::{log_info, logger};

// local import
use config::server;

// local modules
mod api;
mod config;
// mod model;
mod infra;
mod service;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // reload config
    match server::GLOBAL_CONFIG.write().unwrap().reload() {
        Ok(_) => {
            MailManOk::new(200, "config load DONE", None::<&str>);
        }
        Err(e) => {
            panic!("config load error! {e:?}");
        }
    }

    // config out put
    println!("config is {:#?}", &*server::GLOBAL_CONFIG);

    // init logger
    logger::init_logger(
        &server::GLOBAL_CONFIG.read().unwrap().log_path,
        &server::GLOBAL_CONFIG.read().unwrap().log_level,
    );

    // init root PathBuf
    log_info!("Binding Root Dir...");
    let root = PathBuf::from(&server::GLOBAL_CONFIG.read().unwrap().root);

    // init token_cleaner
    // todo!("token Cleaner thread not finish yet !!!");

    // init some config
    log_info!("Server config loading");
    let allowed_origin_list = server::GLOBAL_CONFIG
        .read()
        .unwrap()
        .allowed_origin_list
        .clone();
    // CORS 方法/请求头白名单：启动期解析为强类型，非法配置直接 panic（配置错误应尽早暴露）
    let allowed_methods: Vec<http::Method> = server::GLOBAL_CONFIG
        .read()
        .unwrap()
        .allowed_methods
        .iter()
        .map(|method| {
            method
                .parse()
                .expect("invalid HTTP method in server_config:allowed_methods")
        })
        .collect();
    let allowed_headers: Vec<http::header::HeaderName> = server::GLOBAL_CONFIG
        .read()
        .unwrap()
        .allowed_headers
        .iter()
        .map(|header| {
            http::header::HeaderName::from_bytes(header.as_bytes())
                .expect("invalid header name in server_config:allowed_headers")
        })
        .collect();
    let app_url = format!(
        "{}:{}",
        &*server::GLOBAL_CONFIG.read().unwrap().listen_addr,
        &server::GLOBAL_CONFIG.read().unwrap().listen_port
    );
    let workers = server::GLOBAL_CONFIG.read().unwrap().workers as usize;

    log_info!("HTTP start");
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin_fn({
                        let value = allowed_origin_list.clone();
                        move |origin, _req_head| {
                            value.iter().any(|allowed_origin| origin == allowed_origin)
                        }
                    })
                    .send_wildcard()
                    .allowed_methods(allowed_methods.clone())
                    .allowed_headers(allowed_headers.clone())
                    .max_age(3600),
            )
            // .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(root.clone()))
            // wrap default logger
            .wrap(actix_web::middleware::Logger::default())
            // 统一鉴权中间件（share-lib）：OPTIONS/白名单放行，Bearer 回源 watchman 鉴权，
            // uuid 旧链路比对本机 uuid（individual 模式下无回源字段，仅保留 uuid 比对）
            .wrap(UserAuth::new({
                let config = server::GLOBAL_CONFIG.read().unwrap();
                UserAuthConfig {
                    #[cfg(not(feature = "individual"))]
                    master_addr: config.master_addr.clone(),
                    #[cfg(not(feature = "individual"))]
                    master_port: config.master_port,
                    // 回源自证用子系统名，取自注册名 register_name
                    #[cfg(not(feature = "individual"))]
                    subsys_name: config.register_name.clone(),
                    subsys_uuid: config.subsys_uuid.clone(),
                    authenticate_bypass: config.authenticate_bypass.clone(),
                }
            }))
            .wrap_fn(|req, srv| srv.call(req).map(|res| res))
            .configure(config::app::config_services)
    })
    .workers(workers)
    .bind(&app_url)?
    .run()
    .await
}
