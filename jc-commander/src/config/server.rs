use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::Value;
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

    pub listen_addr: String,
    pub listen_port: u16,
    pub workers: u16,
    pub allowed_origin_list: Vec<String>,

    pub subsys_uuid: String,
    pub register_name: String,

    pub master_addr: String,
    pub master_port: u16,

    pub authenticate_bypass: Vec<String>,

    pub db_str: String,

    pub mq_str: String,
    pub mq_queue_prefix: String,
}

impl AllConfigs {
    pub fn new() -> Self {
        Self {
            log_path: String::new(),
            log_level: String::new(),

            listen_addr: String::new(),
            listen_port: 9001,
            workers: 2,
            allowed_origin_list: vec![],

            subsys_uuid: String::new(),
            register_name: String::new(),

            master_addr: String::new(),
            master_port: 8000,

            authenticate_bypass: vec![],

            db_str: String::new(),

            mq_str: String::new(),
            mq_queue_prefix: String::new(),
        }
    }

    pub fn reload(&mut self) -> Result<u8, MailManErr<'static, String>> {
        let config = match read_config(&mut CONFIG_FILE_HANDLE.lock().unwrap()) {
            Ok(json) => json,
            Err(e) => return Err(e),
        };

        self.log_path = config["server_config"]["log_path"]
            .as_str()
            .expect("Config path server_config:log_path (string) not found")
            .to_string();
        self.log_level = config["server_config"]["log_level"]
            .as_str()
            .expect("Config path server_config:log_level (string) not found")
            .to_string();

        self.listen_addr = config["server_config"]["listen_addr"]
            .as_str()
            .expect("Config path server_config:listen_addr (string) not found")
            .to_string();
        self.listen_port = config["server_config"]["listen_port"]
            .as_u64()
            .expect("Config path server_config:listen_port (u16) not found")
            as u16;
        self.workers = config["server_config"]["workers"]
            .as_u64()
            .expect("Config path server_config:workers (u16) not found")
            as u16;
        self.allowed_origin_list = match &config["server_config"]["allowed_origin_list"] {
            Value::Array(vec) => vec
                .iter()
                .filter_map(|item| item.as_str())
                .map(|item| item.to_string())
                .collect(),
            _ => {
                MailManErr::new(
                    500,
                    "Config Missing",
                    Some("Config path server_config:allowed_origin_list (Array[string]) not found, Using default".to_string()),
                    0,
                );
                vec![
                    "http://localhost:8000".to_string(),
                    "http://127.0.0.1:8000".to_string(),
                ]
            }
        };

        self.subsys_uuid = config["server_config"]["uuid"]
            .as_str()
            .unwrap_or({
                MailManErr::new(
                    500,
                    "Config Missing",
                    Some(
                        "Config path server_config:uuid (string-uuid) not found, Using random"
                            .to_string(),
                    ),
                    0,
                );
                &Uuid::new_v4().to_string()
            })
            .to_string();

        self.register_name = config["server_config"]["register_name"]
            .as_str()
            .expect("Config path server_config:register_name (string) not found")
            .to_string();

        self.master_addr = config["server_config"]["master_addr"]
            .as_str()
            .expect("Config path server_config:master_addr (string) not found")
            .to_string();
        self.master_port = config["server_config"]["master_port"]
            .as_u64()
            .expect("Config path server_config:master_port (u16) not found")
            as u16;

        self.authenticate_bypass = match &config["server_config"]["authenticate_bypass"] {
            Value::Array(vec) => vec
                .iter()
                .filter_map(|item| item.as_str())
                .map(|item| item.to_string())
                .collect(),
            _ => {
                MailManErr::new(
                    500,
                    "Config Missing",
                    Some("server_config:authenticate_bypass (Array[string]) not found, Using default".to_string()),
                    0,
                );
                vec!["/api/refresh_master".to_string()]
            }
        };

        self.db_str = config["server_config"]["db_str"]
            .as_str()
            .expect("Config path server_config:db_str (string) not found")
            .to_string();

        self.mq_str = config["mq_config"]["mq_str"]
            .as_str()
            .expect("Config path mq_config:mq_str (string) not found")
            .to_string();
        self.mq_queue_prefix = config["mq_config"]["queue_prefix"]
            .as_str()
            .expect("Config path mq_config:queue_prefix (string) not found")
            .to_string();

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
