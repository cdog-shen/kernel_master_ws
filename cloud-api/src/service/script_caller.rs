use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::Value;

use share_lib::data_structure::{MailManErr, MailManOk};
use share_lib::infrastructure::process_runner;

use crate::{config::server::GLOBAL_CONFIG, model::cloud_account::CloudAccountModel};

pub async fn run<'a>(
    api_name: &str,
    cloud_user: &str,
    region: &str,
    params: &str,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    let name_list = api_name.split("_").collect::<Vec<&str>>();
    let script_path = GLOBAL_CONFIG.read().unwrap().script_dir.clone()
        + "/"
        + &name_list[0..name_list.len() - 1].join("/")
        + "/"
        + &api_name
        + ".py";
    let mut filter = serde_json::Map::new();
    filter.insert("nick_name".to_string(), serde_json::json!(cloud_user));

    let cloud_user =
        match CloudAccountModel::get_model_with_filter(&filter, &mut pool.get().unwrap()) {
            Ok(data) => {
                if data.is_empty() {
                    return Err(MailManErr::new(
                        400,
                        "Service: run API script - get API Key",
                        Some(format!("CAN NOT find cloud Auth Token: {cloud_user}")),
                        1,
                    ));
                }
                data.clone()
            }
            Err(msg) => match msg.0 {
                1 => {
                    return Err(MailManErr::new(
                        400,
                        "Service: run API script - get API Key",
                        Some(msg.1),
                        1,
                    ));
                }
                _ => {
                    return Err(MailManErr::new(
                        500,
                        "Service: run API script - get API Key",
                        Some(msg.1),
                        1,
                    ));
                }
            },
        };

    let output = process_runner::run(
        &GLOBAL_CONFIG.read().unwrap().python_path,
        &[
            script_path.as_str(),
            cloud_user[0]["ak"].as_str().unwrap(),
            cloud_user[0]["sk"].as_str().unwrap(),
            region,
            params,
        ],
        None,
    );

    let res = match output {
        Ok(output) => {
            // 标准输出
            if !output.stdout.is_empty() {
                Ok(output.stdout)
            } else if !output.stderr.is_empty() {
                Ok(output.stderr)
            } else {
                Err("Nothing in stdout".to_string())
            }
        }
        Err(e) => Err(format!("Error: {e:?}")),
    };

    match res {
        Ok(res_data) => match serde_json::from_str::<Value>(&res_data) {
            Ok(json_value) => Ok(MailManOk::new(
                200,
                "Service: run API script",
                Some(json_value),
            )),
            Err(_) => Err(MailManErr::new(
                500,
                "Service: run API script",
                Some(format!("Output JSON parse error: {res_data}")),
                1,
            )),
        },
        Err(err_data) => Err(MailManErr::new(
            500,
            "Service: run API script",
            Some(err_data),
            1,
        )),
    }
}

pub async fn get_scripts<'a>(
    provider_name: &str,
    product_name: &str,
) -> Result<MailManOk<'a, Value>, MailManErr<'a, String>> {
    let script_path =
        GLOBAL_CONFIG.read().unwrap().script_dir.clone() + "/" + provider_name + "/" + product_name;

    let res = match std::fs::read_dir(&script_path) {
        Ok(entries) => {
            let mut names = Vec::new();
            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(e) => {
                        return Err(MailManErr::new(
                            500,
                            "Service: get Product script",
                            Some(format!("Error: {e}")),
                            1,
                        ));
                    }
                };
                names.push(entry.file_name().to_string_lossy().into_owned());
            }
            // 与 `ls <dir>` 输出等价：按文件名排序，逐行换行
            names.sort_unstable();
            if names.is_empty() {
                Err("Nothing in stdout".to_string())
            } else {
                Ok(names.join("\n") + "\n")
            }
        }
        Err(e) => Err(format!("Error: {e}")),
    };

    match res {
        Ok(res_data) => Ok(MailManOk::new(
            200,
            "Get success",
            Some(serde_json::to_value(res_data).unwrap()),
        )),
        Err(err_data) => Err(MailManErr::new(
            500,
            "Service: get Product script",
            Some(err_data.to_string()),
            1,
        )),
    }
}
