// std import
use crossbeam::queue::SegQueue;
use std::default::Default;
use std::io;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
// rt import
use actix_cors::Cors;
use actix_web::dev::Service;
use actix_web::web;
use actix_web::{http, App, HttpServer};
use futures::FutureExt;
// db utils import
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
// mq utils import
use lapin::{Connection, ConnectionProperties};
// db models

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

    // init mysql connection pool
    let db_manager =
        ConnectionManager::<PgConnection>::new(&*server::GLOBAL_CONFIG.read().unwrap().db_str);
    let db_pool = diesel::r2d2::Pool::builder()
        .build(db_manager)
        .expect("Failed to create pool.");

    // init MQ connection pool
    let mq_str = server::GLOBAL_CONFIG.read().unwrap().mq_str.clone();
    let mq_manager = Connection::connect(&mq_str, ConnectionProperties::default())
        .await
        .expect("Failed to connect to MQ");
    let mq_pool: Arc<Connection> = Arc::new(mq_manager);

    // init DoneTaskList
    log::info!("Creating DoneTaskList...");
    let done_task_list: Arc<SegQueue<Uuid>> = Arc::new(SegQueue::new());

    // initialize the scheduled task scheduler and start Ticking
    let time_wheel = Arc::new(util::scheduler::TimeWheel::new(
        db_pool.clone(),
        mq_pool.clone(),
    ));
    let flush_flag = Arc::new(Mutex::new(false));
    let ffc = flush_flag.clone();
    let _ = time_wheel.reload_from_db();
    tokio::spawn(async move {
        log::info!("TimeWheel thread launched. Ticking...");
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            if *ffc.lock().unwrap() {
                *ffc.lock().unwrap() = false;
                let _ = time_wheel.reload_from_db();
            }
            interval.tick().await;
            if time_wheel.tick() {
                *ffc.lock().unwrap() = true;
            }
        }
    });

    // init some config
    let allowed_origin_list = server::GLOBAL_CONFIG
        .read()
        .unwrap()
        .allowed_origin_list
        .clone();
    let app_url = format!(
        "{}:{}",
        &*server::GLOBAL_CONFIG.read().unwrap().listen_addr,
        &server::GLOBAL_CONFIG.read().unwrap().listen_port
    );
    let workers = server::GLOBAL_CONFIG.read().unwrap().workers as usize;

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
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(mq_pool.clone()))
            .app_data(web::Data::from(flush_flag.clone()))
            .app_data(web::Data::from(done_task_list.clone()))
            // wrap default logger
            .wrap(actix_web::middleware::Logger::default())
            // Comment this line if you want to integrate with yew-address-book-frontend
            .wrap(crate::middleware::auth_middleware::Authentication)
            .wrap_fn(|req, srv| srv.call(req).map(|res| res))
            .configure(config::app::config_services)
    })
    .workers(workers)
    .bind(&app_url)?
    .run()
    .await
}
