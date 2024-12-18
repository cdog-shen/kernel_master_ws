use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::{
    cell::RefCell,
    fs::{read_to_string, File},
    sync::{Mutex, RwLock},
};

use share_lib::{cfg_reader::read_config, data_structure::MailManErr};

#[derive(Debug, Deserialize)]
pub struct AllConfigs {
    pub log_path: String,
    pub log_level: String,

    pub db_str: String,

    pub listen_addr: String,
    pub listen_port: u16,

    pub pub_key_path: String,
    pub pri_key_path: String,
    pub secret_key_path: String,

    pub authenticate_bypass: Vec<String>,
    pub permit_bypass: Vec<String>,
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
            listen_addr: String::new(),
            listen_port: 8000,
            pub_key_path: String::new(),
            pri_key_path: String::new(),
            secret_key_path: String::new(),
            authenticate_bypass: vec![],
            permit_bypass: vec![],
        }
    }

    pub fn reload(&mut self) -> Result<u8, MailManErr<'static>> {
        let config = match read_config(&mut *CONFIG_FILE_HANDLE.lock().unwrap()) {
            Ok(json) => json,
            Err(e) => return Err(e),
        };

        println!("{:?}", config["server_config"]["log_path"]);

        self.log_path = get_string_from_config(&config, &["server_config", "log_path"]);
        self.db_str = get_string_from_config(&config, &["db_config", "db_str"]);
        self.listen_addr = get_string_from_config(&config, &["server_config", "listen_addr"]);
        self.listen_port = config["server_config"]["listen_port"].as_u64().unwrap() as u16;
        self.pub_key_path = get_string_from_config(&config, &["server_config", "pub_key_path"]);
        self.pri_key_path = get_string_from_config(&config, &["server_config", "pri_key_path"]);
        self.secret_key_path = get_string_from_config(&config, &["server_config", "secret_key_path"]);
        self.authenticate_bypass = match &config["server_config"]["authenticate_bypass"] {
            Value::Array(vec) => vec
                .into_iter()
                .filter_map(|item| item.as_str())
                .filter_map(|item| Some(item.to_string()))
                .collect(),
            _ => vec![],
        };
        self.permit_bypass = match &config["server_config"]["permit_bypass"] {
            Value::Array(vec) => vec
                .into_iter()
                .filter_map(|item| item.as_str())
                .filter_map(|item| Some(item.to_string()))
                .collect(),
            _ => vec![],
        };

        Ok(0)
    }
}

pub static CONFIG_FILE_HANDLE: Lazy<Mutex<File>> = Lazy::new(|| {
    let path = std::env::current_dir()
        .expect("Unable to get workspace path")
        .join("watchman_server.cfg");
    let file = File::open(&path).expect("Unable to open config file");
    Mutex::new(file)
});

pub static GLOBAL_CONFIG: Lazy<RwLock<AllConfigs>> = Lazy::new(|| RwLock::new(AllConfigs::new()));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_config_success() {
        let config: Result<Map<String, Value>, share_lib::data_structure::MailManErr<'_>> =
            read_config(&mut *CONFIG_FILE_HANDLE.lock().unwrap());

        // println!("{:?}", config);
        assert!(config.is_ok());

        let config = config.unwrap();

        println!("{:?}", config);
        assert_eq!(config["server_config"]["listen_port"], 8000);

        let config2 = read_config(&mut *CONFIG_FILE_HANDLE.lock().unwrap());

        assert!(config2.is_ok());

        let config2 = config2.unwrap();
        println!("{:?}", config2);
        assert_eq!(config2["server_config"]["listen_port"], 8000);
    }
}
