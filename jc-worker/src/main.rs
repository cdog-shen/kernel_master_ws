// share-lib import
use share_lib::data_structure::{MailManErr, MailManOk};
use share_lib::logger;
// local import
use config::worker;
// local modules
mod config;
mod mq_consumer;
mod service;
mod util;

#[tokio::main]
async fn main() {
    // reload config
    match worker::GLOBAL_CONFIG.write().unwrap().reload() {
        Ok(_) => {
            MailManOk::new(200, "config load DONE", None::<&str>);
        }
        Err(e) => {
            MailManErr::new(500, "config load Failed", Some(e), 2);
            panic!("config load Failed");
        }
    }

    // config out put
    println!("config is {:#?}", &*worker::GLOBAL_CONFIG);

    // init logger
    logger::init_logger(
        &worker::GLOBAL_CONFIG.read().unwrap().log_path,
        &worker::GLOBAL_CONFIG.read().unwrap().log_level,
    );

    // start MQ consume loop
    mq_consumer::run().await;
}
