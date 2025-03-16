use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::{
    fs::File,
    sync::{Mutex, RwLock},
};
use uuid::Uuid;

use share_lib::{cfg_reader::read_config, data_structure::MailManErr};

#[derive(Debug, Deserialize)]
pub struct AllConfigs {
    pub log_path: String,
    pub log_level: String,

    pub db_str: String,
    pub mq_str: String,
    pub mq_queue_prefix: String,

    pub listen_addr: String,
    pub listen_port: u16,

    pub authenticate_bypass: Vec<String>,
    pub subsys_uuid: String,

    pub master_addr: String,
    pub master_port: u16,
    pub register_name: String,

    pub python_path: String,
    pub script_dir: String,
}

fn get_string_from_config(config: &Map<String, Value>, path: &[&str]) -> String {
    match config[path[0]][path[1]].as_str() {
        Some(data) => data.to_string(),
        None => "".to_string(),
    }
}

impl AllConfigs {
    pub fn new() -> Self {
        Self {
            log_path: String::new(),
            log_level: String::new(),
            db_str: String::new(),
            mq_str: String::new(),
            mq_queue_prefix: String::new(),
            listen_addr: String::new(),
            listen_port: 9001,
            authenticate_bypass: vec![],
            subsys_uuid: String::new(),
            master_addr: String::new(),
            master_port: 8000,
            register_name: String::new(),
            python_path: String::new(),
            script_dir: String::new(),
        }
    }

    pub fn reload(&mut self) -> Result<u8, MailManErr<'static>> {
        let config = match read_config(&mut CONFIG_FILE_HANDLE.lock().unwrap()) {
            Ok(json) => json,
            Err(e) => return Err(e),
        };

        println!("{:?}", config["server_config"]["log_path"]);

        self.log_path = get_string_from_config(&config, &["server_config", "log_path"]);
        self.db_str = get_string_from_config(&config, &["db_config", "db_str"]);
        self.mq_str = get_string_from_config(&config, &["mq_config", "mq_str"]);
        self.mq_queue_prefix = get_string_from_config(&config, &["mq_config", "queue_prefix"]);
        self.listen_addr = get_string_from_config(&config, &["server_config", "listen_addr"]);
        self.listen_port = config["server_config"]["listen_port"].as_u64().unwrap() as u16;
        self.authenticate_bypass = match &config["server_config"]["authenticate_bypass"] {
            Value::Array(vec) => vec
                .iter()
                .filter_map(|item| item.as_str())
                .map(|item| item.to_string())
                .collect(),
            _ => vec![],
        };
        self.subsys_uuid = Uuid::new_v4().to_string();
        self.master_addr = get_string_from_config(&config, &["server_config", "master_addr"]);
        self.master_port = config["server_config"]["master_port"].as_u64().unwrap() as u16;
        self.register_name = get_string_from_config(&config, &["server_config", "register_name"]);
        self.python_path = get_string_from_config(&config, &["server_config", "python_path"]);
        self.script_dir = get_string_from_config(&config, &["server_config", "script_dir"]);

        Ok(0)
    }
}

pub static CONFIG_FILE_HANDLE: Lazy<Mutex<File>> = Lazy::new(|| {
    let path = std::env::current_dir()
        .expect("Unable to get workspace path")
        .join("job_center_commander.cfg");
    let file = File::open(&path).expect("Unable to open config file");
    Mutex::new(file)
});

pub static GLOBAL_CONFIG: Lazy<RwLock<AllConfigs>> = Lazy::new(|| RwLock::new(AllConfigs::new()));
