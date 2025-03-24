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

    pub subsys_uuid: String,

    pub commander_addr: String,
    pub commander_port: u16,

    pub python_path: String,
    pub script_dir: String,

    pub mq_str: String,
    pub mq_queue_prefix: String,
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

            subsys_uuid: String::new(),

            commander_addr: String::new(),
            commander_port: 9003,

            python_path: String::new(),
            script_dir: String::new(),

            mq_str: String::new(),
            mq_queue_prefix: String::new(),
        }
    }

    pub fn reload(&mut self) -> Result<u8, MailManErr<'static>> {
        let config = match read_config(&mut CONFIG_FILE_HANDLE.lock().unwrap()) {
            Ok(json) => json,
            Err(e) => return Err(e),
        };

        self.log_path = get_string_from_config(&config, &["server_config", "log_path"]);
        self.log_level = get_string_from_config(&config, &["server_config", "log_level"]);

        self.subsys_uuid = Uuid::new_v4().to_string();

        self.commander_addr = get_string_from_config(&config, &["server_config", "commander_addr"]);
        self.commander_port = match config["server_config"]["commander_port"].as_u64() {
            Some(data) => data as u16,
            None => 9003,
        };

        self.python_path = get_string_from_config(&config, &["server_config", "python_path"]);
        self.script_dir = get_string_from_config(&config, &["server_config", "script_dir"]);

        self.mq_str = get_string_from_config(&config, &["mq_config", "mq_str"]);
        self.mq_queue_prefix = get_string_from_config(&config, &["mq_config", "queue_prefix"]);

        Ok(0)
    }
}

pub static CONFIG_FILE_HANDLE: Lazy<Mutex<File>> = Lazy::new(|| {
    let path = std::env::current_dir()
        .expect("Unable to get workspace path")
        .join("job_center_worker.cfg");
    let file = File::open(&path).expect("Unable to open config file");
    Mutex::new(file)
});

pub static GLOBAL_CONFIG: Lazy<RwLock<AllConfigs>> = Lazy::new(|| RwLock::new(AllConfigs::new()));
