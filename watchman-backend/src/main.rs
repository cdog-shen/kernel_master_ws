// #![allow(unused_must_use)]

// std import
use std::default::Default;
use std::{env, io};
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
use share_lib::cfg_reader::GLOBAL_CONFIG_HANDLER;

// local modules
mod middleware;

#[actix_rt::main]
async fn main() {
    // config out put
    println!("config is {:#?}", &*GLOBAL_CONFIG_HANDLER);

    // init logger
    // share_lib::logger::init_logger(&*GLOBAL_CONFIG_HANDLER.server_config.log_level);

    // init mysql connection pool
    let manager =
        ConnectionManager::<MysqlConnection>::new(GLOBAL_CONFIG_HANDLER.db_config.db_str.clone());
    let pool = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

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
            // wrap custom looger from share-lib logger
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
