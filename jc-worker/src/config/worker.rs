use once_cell::sync::Lazy;
use serde::Deserialize;
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

    pub fn reload(&mut self) -> Result<u8, MailManErr<'static, String>> {
        let config = read_config(&mut CONFIG_FILE_HANDLE.lock().unwrap())?;

        self.log_path = config["server_config"]["log_path"]
            .as_str()
            .expect("Config path server_config:log_path (string) not found")
            .to_string();
        self.log_level = config["server_config"]["log_level"]
            .as_str()
            .expect("Config path server_config:log_level (string) not found")
            .to_string();

        let uuid = Uuid::new_v4().to_string();
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
                &uuid
            })
            .to_string();

        self.commander_addr = config["server_config"]["commander_addr"]
            .as_str()
            .expect("Config path server_config:commander_addr (string) not found")
            .to_string();
        self.commander_port = config["server_config"]["commander_port"]
            .as_u64()
            .expect("Config path server_config:commander_port (u16) not found")
            as u16;

        self.python_path = config["server_config"]["python_path"]
            .as_str()
            .expect("Config path server_config:python_path (string) not found")
            .to_string();
        self.script_dir = config["server_config"]["script_dir"]
            .as_str()
            .expect("Config path server_config:script_dir (string) not found")
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
        .join("job_center_worker.cfg");
    let file = File::open(&path).expect("Unable to open config file");
    Mutex::new(file)
});

pub static GLOBAL_CONFIG: Lazy<RwLock<AllConfigs>> = Lazy::new(|| RwLock::new(AllConfigs::new()));
