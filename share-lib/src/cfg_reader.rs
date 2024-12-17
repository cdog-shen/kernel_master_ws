use serde_json::{Map, Value};
use std::fs;

use crate::data_structure;

pub fn read_config<P>(path: P) -> Result<Map<String, Value>, data_structure::MailManErr<'static>>
where
    P: AsRef<std::path::Path> + std::fmt::Display,
{
    let path_str = path.to_string();

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

    let config: Map<String, Value> = match toml::de::from_str(&cfg_content) {
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

    data_structure::MailManOk::new(
        0,
        "Config file load done",
        Some(format!("path is : {}", path)),
    );

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_config_success() {
        let path = "Cargo.toml";
        let result = read_config(path);
        assert!(result.is_ok());
        let config = result.unwrap();

        println!("{:?}", config);

        assert_eq!(config["package"]["name"], "share-lib");
    }
}
