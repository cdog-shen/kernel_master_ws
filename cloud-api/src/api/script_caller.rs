use actix_web::{web, HttpResponse};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::Value;

use share_lib::data_structure::MailManOk;

use crate::server::GLOBAL_CONFIG;
use crate::services::account_service;

// run cloud api script
pub async fn run(
    req: web::Json<Value>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> HttpResponse {
    let api_name_str = req["api_name"].as_str().unwrap().to_string();
    let name_list = api_name_str.split("_").into_iter().collect::<Vec<&str>>();
    let script_path = GLOBAL_CONFIG.read().unwrap().script_dir.clone()
        + "/"
        + &name_list[0..name_list.len() - 1].join("/")
        + "/"
        + &api_name_str
        + ".py";
    let mut filter = serde_json::Map::new();
    filter.insert(
        "nick_name".to_string(),
        serde_json::Value::String(req["nick_name"].as_str().unwrap().to_string()),
    );

    let cloud_access = match account_service::get_all(&filter, &pool) {
        Ok(data) => {
            let data_unwrapped = data.data.unwrap();
            if data_unwrapped.len() == 0 {
                return HttpResponse::BadRequest().json("No account found");
            }
            data_unwrapped.clone()
        }
        Err(err) => return HttpResponse::BadRequest().json(err),
    };

    let output = std::process::Command::new(GLOBAL_CONFIG.read().unwrap().python_path.clone())
        .arg(script_path)
        .arg(cloud_access[0]["AK"].as_str().unwrap())
        .arg(cloud_access[0]["SK"].as_str().unwrap())
        .arg(req["region"].as_str().unwrap())
        .arg(req["params"].as_str().unwrap())
        .output();

    let res = match output {
        Ok(output) => {
            // 标准输出
            if !output.stdout.is_empty() {
                Ok(String::from_utf8(output.stdout).expect("Error: stdout is not utf8"))
            } else if !output.stderr.is_empty() {
                Ok(String::from_utf8(output.stderr).expect("Error: stdout is not utf8"))
            } else {
                Err("Nothing in stdout".to_string())
            }
        }
        Err(e) => Err(format!("Error: {}", e)),
    };

    match res {
        Ok(res_data) => HttpResponse::Ok().json(MailManOk::new(
            200,
            "Call success",
            Some(serde_json::to_value(res_data).unwrap()),
        )),
        Err(err_data) => HttpResponse::InternalServerError().json(err_data.to_string()),
    }
}

// get cloud api script
pub async fn get(req: web::Json<Value>) -> HttpResponse {
    let script_path = GLOBAL_CONFIG.read().unwrap().script_dir.clone()
        + "/"
        + req["provider_name"].as_str().unwrap()
        + "/"
        + req["product_name"].as_str().unwrap();
    let output = std::process::Command::new("ls").arg(script_path).output();

    let res = match output {
        Ok(output) => {
            // 标准输出
            if !output.stdout.is_empty() {
                Ok(String::from_utf8(output.stdout).expect("Error: stdout is not utf8"))
            } else if !output.stderr.is_empty() {
                Ok(String::from_utf8(output.stderr).expect("Error: stdout is not utf8"))
            } else {
                Err("Nothing in stdout".to_string())
            }
        }
        Err(e) => Err(format!("Error: {}", e)),
    };

    match res {
        Ok(res_data) => HttpResponse::Ok().json(MailManOk::new(
            200,
            "Get success",
            Some(serde_json::to_value(res_data).unwrap()),
        )),
        Err(err_data) => HttpResponse::InternalServerError().json(err_data.to_string()),
    }
}
