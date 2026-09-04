use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::Value;
use std::{
    fs::{File, read_to_string},
    sync::{Mutex, RwLock},
};

use share_lib::{cfg_reader::read_config, data_structure::MailManErr};

#[derive(Debug, Deserialize)]
pub struct AllConfigs {
    pub log_path: String,
    pub log_level: String,

    pub listen_addr: String,
    pub listen_port: u16,
    pub workers: u16,
    pub allowed_origin_list: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,

    // pub pub_key_path: String,
    // pub pri_key_path: String,
    pub secret_key_path: String,

    pub authenticate_bypass: Vec<String>,
    pub permit_bypass: Vec<String>,

    pub db_str: String,
}

impl AllConfigs {
    pub fn new() -> Self {
        Self {
            log_path: String::new(),
            log_level: String::new(),

            listen_addr: String::new(),
            listen_port: 8000,
            workers: 2,
            allowed_origin_list: vec![],
            // CORS 默认放行方法（覆盖全仓库路由实际用到的方法 + OPTIONS 预检）与请求头（实际只用到这两个）
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PATCH".to_string(),
                "DELETE".to_string(),
                "PUT".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],

            // pub_key_path: String::new(),
            // pri_key_path: String::new(),
            secret_key_path: String::new(),

            authenticate_bypass: vec![],
            permit_bypass: vec![],

            db_str: String::new(),
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
                    Some("server_config:allowed_origin_list (Array[string]) not found, Using default".to_string()),
                    0,
                );
                vec![
                    "http://localhost:3000".to_string(),
                    "http://127.0.0.1:3000".to_string(),
                ]
            }
        };
        self.allowed_methods = match &config["server_config"]["allowed_methods"] {
            Value::Array(vec) => vec
                .iter()
                .filter_map(|item| item.as_str())
                .map(|item| item.to_string())
                .collect(),
            _ => {
                MailManErr::new(
                    500,
                    "Config Missing",
                    Some("Config path server_config:allowed_methods (Array[string]) not found, Using default".to_string()),
                    0,
                );
                vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PATCH".to_string(),
                    "DELETE".to_string(),
                    "PUT".to_string(),
                    "OPTIONS".to_string(),
                ]
            }
        };
        self.allowed_headers = match &config["server_config"]["allowed_headers"] {
            Value::Array(vec) => vec
                .iter()
                .filter_map(|item| item.as_str())
                .map(|item| item.to_string())
                .collect(),
            _ => {
                MailManErr::new(
                    500,
                    "Config Missing",
                    Some("Config path server_config:allowed_headers (Array[string]) not found, Using default".to_string()),
                    0,
                );
                vec!["Content-Type".to_string(), "Authorization".to_string()]
            }
        };

        // self.pub_key_path = config["server_config"]["pub_key_path"]
        //     .as_str()
        //     .expect("Config path server_config:pub_key_path (string) not found")
        //     .to_string();
        // self.pri_key_path = config["server_config"]["pri_key_path"]
        //     .as_str()
        //     .expect("Config path server_config:pri_key_path (string) not found")
        //     .to_string();
        self.secret_key_path = config["server_config"]["secret_key_path"]
            .as_str()
            .expect("Config path server_config:secret_key_path (string) not found, Using Empty")
            .to_string();

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
                vec![
                    "/api/hey".to_string(),
                    "/webhook".to_string(),
                    "/api/auth/login".to_string(),
                    "/api/auth/signup".to_string(),
                    "/api/subsystem_control/all_subsystem".to_string(),
                    "/api/subsystem_control/update_subsystem".to_string(),
                ]
            }
        };
        self.permit_bypass = match &config["server_config"]["permit_bypass"] {
            Value::Array(vec) => vec
                .iter()
                .filter_map(|item| item.as_str())
                .map(|item| item.to_string())
                .collect(),
            _ => {
                MailManErr::new(
                    500,
                    "Config Missing",
                    Some(
                        "server_config:permit_bypass (Array[string]) not found, Using default"
                            .to_string(),
                    ),
                    0,
                );
                vec![
                    "/webhook".to_string(),
                    "/api/hey".to_string(),
                    "/api/auth/login".to_string(),
                    "/api/auth/signup".to_string(),
                    // 回源鉴权端点自身跳过权限判定（JWT 仍须验）：
                    // 真正判定的是 body 里的目标 path/method
                    "/api/auth/verify".to_string(),
                    "/api/subsystem_control/all_subsystem".to_string(),
                    "/api/subsystem_control/update_subsystem".to_string(),
                ]
            }
        };

        self.db_str = config["db_config"]["db_str"]
            .as_str()
            .expect("Config path db_config:db_str (string) not found")
            .to_string();

        Ok(0)
    }
}

pub static CONFIG_FILE_HANDLE: Lazy<Mutex<File>> = Lazy::new(|| {
    let path = std::env::current_dir()
        .expect("Unable to get workspace path")
        .join("watchman.toml");
    let file = File::open(&path).expect("Unable to open config file");
    Mutex::new(file)
});

pub static GLOBAL_CONFIG: Lazy<RwLock<AllConfigs>> = Lazy::new(|| RwLock::new(AllConfigs::new()));
pub static SECRET_KEY: Lazy<RwLock<String>> = Lazy::new(|| {
    RwLock::new({
        let secret_path = &GLOBAL_CONFIG.read().unwrap().secret_key_path;
        match read_to_string(secret_path) {
            Ok(key) => key,
            Err(e) => {
                MailManErr::new(500, "SECRET key read error :", Some(e), 1);
                "".to_string()
            }
        }
    })
});
