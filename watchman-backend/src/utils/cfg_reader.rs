use std::{borrow::Cow, fs};
use serde::Deserialize;

use share_lib::data_structure;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub listen_addr: Cow<'static, str>,
    pub listen_port: u16,
    pub log_path: Cow<'static, str>,
    pub log_level: Cow<'static, str>,
    // pub pub_key_path: Option<Cow<'static, str>>,
    // pub pri_key_path: Option<Cow<'static, str>>,
    pub secret_key_path: Option<Cow<'static, str>>,
}

#[derive(Debug, Deserialize)]
pub struct DBConfig {
    pub db_str: Cow<'static, str>,
}

// #[derive(Debug, Deserialize)]
// pub struct KeyKeeper {
//     pub pub_key: Cow<'static, str>,
//     pub pri_key: Cow<'static, str>,
// }

#[derive(Debug, Deserialize)]
pub struct SubSysConfig {
    // temporary keep
}

#[derive(Debug, Deserialize)]
pub struct ConfigKeeper {
    pub config_path: Option<Cow<'static, str>>,
    pub server_config: ServerConfig,
    pub db_config: DBConfig,
    // subsys_config: SubSysConfig,
}

impl ConfigKeeper {
    pub fn new<P>(path: P) -> Result<Self, data_structure::MailManErr<'static>>
    where
        P: AsRef<std::path::Path> + std::fmt::Display,
    {
        let path_str = path.to_string(); // 先将 `path` 转换为 `String`

        let cfg_content = match fs::read_to_string(&path_str) {
            Ok(content) => {
                data_structure::MailManOk::new(0, "Config file read success", None::<&str>);
                content
            }
            Err(e) => {
                return Err(data_structure::MailManErr::new(
                    500,
                    "Config loading - failed to open file",
                    e,
                    2,
                ));
            }
        };

        let mut config: ConfigKeeper = match toml::de::from_str(&cfg_content) {
            Ok(parsed) => parsed,
            Err(e) => {
                return Err(data_structure::MailManErr::new(
                    500,
                    "Config loading - Unable to parse CFG",
                    e,
                    2,
                ));
            }
        };

        // 在这里存储 `path_str` 的引用，这样避免了所有权问题
        config.config_path = Some(Cow::Owned(path_str.clone()));

        data_structure::MailManOk::new(
            0,
            "Config file load done",
            Some(format!("path is : {}", path)),
        );

        Ok(config)
    }
}

// load config file

pub static GLOBAL_CONFIG_HANDLER: once_cell::sync::Lazy<ConfigKeeper> =
    once_cell::sync::Lazy::new(|| {
        let config_path = std::env::current_dir()
            .expect("Unable to get workspace path")
            .join("watchman_server.cfg")
            .to_str()
            .expect("Failed to convert path to str")
            .to_string();
        println!("now we are at : {}", &config_path);
        ConfigKeeper::new(config_path.clone()).unwrap()
    });

// pub static PUB_KEY: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
//     let pub_path = match &GLOBAL_CONFIG_HANDLER.server_config.pub_key_path {
//         Some(_) => GLOBAL_CONFIG_HANDLER
//             .server_config
//             .pub_key_path
//             .as_ref()
//             .unwrap(),
//         None => {
//             data_structure::MailManErr::new(
//                 500,
//                 "PUB key config missing :",
//                 "GLOBAL_CONFIG_HANDLER.server_config.pub_key_path is None",
//                 0,
//             );
//             ""
//         }
//     };

//     match fs::read_to_string(pub_path) {
//         Ok(key) => key,
//         Err(e) => {
//             data_structure::MailManErr::new(500, "PUB key read error :", e, 1);
//             "".to_string()
//         }
//     }
// });

// pub static PRI_KEY: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
//     let pri_path = match &GLOBAL_CONFIG_HANDLER.server_config.pri_key_path {
//         Some(_) => GLOBAL_CONFIG_HANDLER
//             .server_config
//             .pri_key_path
//             .as_ref()
//             .unwrap(),
//         None => {
//             data_structure::MailManErr::new(
//                 500,
//                 "PRI key config missing :",
//                 "GLOBAL_CONFIG_HANDLER.server_config.pri_key_path is None",
//                 0,
//             );
//             ""
//         }
//     };

//     match fs::read_to_string(pri_path) {
//         Ok(key) => key,
//         Err(e) => {
//             data_structure::MailManErr::new(500, "PRI key read error :", e, 1);
//             "".to_string()
//         }
//     }
// });

pub static SECRET_KEY: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
    let secret_path = match &GLOBAL_CONFIG_HANDLER.server_config.secret_key_path {
        Some(_) => GLOBAL_CONFIG_HANDLER
            .server_config
            .secret_key_path
            .as_ref()
            .unwrap(),
        None => {
            data_structure::MailManErr::new(
                500,
                "SECRET key config missing :",
                "GLOBAL_CONFIG_HANDLER.server_config.secret_key_path is None",
                0,
            );
            ""
        }
    };

    match fs::read_to_string(secret_path) {
        Ok(key) => key,
        Err(e) => {
            data_structure::MailManErr::new(500, "SECRET key read error :", e, 1);
            "".to_string()
        }
    }
});
