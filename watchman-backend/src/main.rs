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

#[actix_rt::main]
async fn main() {
    // config out put
    println!("config is {:#?}", &*GLOBAL_CONFIG_HANDLER);

    // init logger
    share_lib::logger::init_logger(&*GLOBAL_CONFIG_HANDLER.server_config.log_level);

    // init mysql connection pool
    let manager =
        ConnectionManager::<MysqlConnection>::new(GLOBAL_CONFIG_HANDLER.db_config.db_str.clone());
    let pool = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
}
