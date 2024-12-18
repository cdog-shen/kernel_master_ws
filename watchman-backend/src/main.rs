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
// db models

// share-lib import
use share_lib;
// local import
use config::server;
use share_lib::data_structure::MailManOk;
use utils::cfg_reader::GLOBAL_CONFIG_HANDLER;

// local modules
mod api;
mod config;
mod middleware;
mod models;
mod services;
mod utils;

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
    println!("config is {:#?}", &*server::GLOBAL_CONFIG);

    // init logger
    share_lib::logger::init_logger(
        &server::GLOBAL_CONFIG.read().unwrap().log_path,
        &server::GLOBAL_CONFIG.read().unwrap().log_level,
    );

    // init mysql connection pool
    let manager =
        ConnectionManager::<MysqlConnection>::new(&*server::GLOBAL_CONFIG.read().unwrap().db_str);
    let pool = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    // setting listen addr
    let app_url = format!(
        "{}:{}",
        &*server::GLOBAL_CONFIG.read().unwrap().listen_addr,
        &server::GLOBAL_CONFIG.read().unwrap().listen_port
    );

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default() // allowed_origin return access-control-allow-origin: * by default
                    .allowed_origin("http://127.0.0.1:3000")
                    .allowed_origin("http://localhost:3000")
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
    .bind(&app_url)?
    .run()
    .await
}
