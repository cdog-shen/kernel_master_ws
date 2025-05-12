// #![allow(unused_must_use)]

// std import
use std::default::Default;
use std::io;
// rt import
use actix_cors::Cors;
use actix_web::dev::Service;
use actix_web::web;
use actix_web::{http, App, HttpServer};
use futures::FutureExt;
// db utils import
use diesel::r2d2::ConnectionManager;
use diesel::MysqlConnection;

// share-lib import
use share_lib::data_structure::MailManOk;
use share_lib::logger;

// local import
use config::server;

// local modules
mod api;
mod config;
mod middleware;
mod model;
mod service;
mod util;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    // reload config
    match server::GLOBAL_CONFIG.write().unwrap().reload() {
        Ok(_) => {
            MailManOk::new(200, "config load DONE", None::<&str>);
        }
        Err(e) => {
            panic!("config load error! {}", e);
        }
    }

    // config out put
    println!("config is {:#?}", &server::GLOBAL_CONFIG);

    // init logger
    logger::init_logger(
        &server::GLOBAL_CONFIG.read().unwrap().log_path,
        &server::GLOBAL_CONFIG.read().unwrap().log_level,
    );

    // init mysql connection pool
    let manager =
        ConnectionManager::<MysqlConnection>::new(&server::GLOBAL_CONFIG.read().unwrap().db_str);
    let pool = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    // setting listen addr
    let app_url = format!(
        "{}:{}",
        &server::GLOBAL_CONFIG.read().unwrap().listen_addr,
        &server::GLOBAL_CONFIG.read().unwrap().listen_port
    );
    let allowed_origin_list = server::GLOBAL_CONFIG
        .read()
        .unwrap()
        .allowed_origin_list
        .clone();

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default() // allowed_origin return access-control-allow-origin: * by default
                    .allowed_origin_fn({
                        let value = allowed_origin_list.clone();
                        move |origin, _req_head| {
                            value.iter().any(|allowed_origin| origin == allowed_origin)
                        }
                    })
                    .send_wildcard()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .app_data(web::Data::new(pool.clone()))
            // wrap default logger
            .wrap(actix_web::middleware::Logger::default())
            // Comment this line if you want to integrate with yew-address-book-frontend
            .wrap(crate::middleware::auth_middleware::Authentication)
            .wrap_fn(|req, srv| srv.call(req).map(|res| res))
            .configure(config::app::config_services)
    })
    .workers(server::GLOBAL_CONFIG.read().unwrap().workers as usize)
    .bind(&app_url)?
    .run()
    .await
}
