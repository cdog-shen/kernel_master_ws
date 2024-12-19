use serde_json::{Map, Value};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use crate::data_structure;

pub fn read_config(
    file_ptr: &mut File,
) -> Result<Map<String, Value>, data_structure::MailManErr<'static>> {
    match file_ptr.seek(SeekFrom::Start(0)) {
        Ok(_) => {
            data_structure::MailManOk::new(0, "Config file reset success", None::<&str>);
            ()
        }
        Err(e) => {
            return Err(data_structure::MailManErr::new(
                500,
                "Config loading - failed to reset file",
                e,
                2,
            ));
        }
    }
    let mut file_contents = String::new();

    let _cfg_length = match file_ptr.read_to_string(&mut file_contents) {
        Ok(content) => {
            data_structure::MailManOk::new(
                0,
                "Config file read success",
                Some(format!("length: {}", content)),
            );
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

    let config: Map<String, Value> = match toml::de::from_str(&file_contents) {
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
        "Config file reload done",
        Some(format!("path is : {:?}", file_ptr)),
    );

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_config_success() {
        let path = "Cargo.toml";
        let mut file = std::fs::File::open(&path).expect("Unable to open config file");
        let result = read_config(&mut file);

        println!("{:?}", result);
        assert!(result.is_ok());

        let config = result.unwrap();

        println!("{:?}", config);
        assert_eq!(config["package"]["name"], "share-lib");
    }
}
