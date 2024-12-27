use actix_web::{web, HttpResponse};
use serde_json::Value;

use share_lib::data_structure::MailManOk;

use crate::server::GLOBAL_CONFIG;

// run cloud api script
pub async fn run(req: web::Json<Value>) -> HttpResponse {
    // let res: Result<String, String> = Ok(format!("need to impl. data: {}", req.to_string()));
    let api_name_str = req["api_name"].as_str().unwrap().to_string();
    let name_list = api_name_str.split("_").into_iter().collect::<Vec<&str>>();
    let script_path = GLOBAL_CONFIG.read().unwrap().script_dir.clone()
        + "/"
        + &name_list[0..name_list.len() - 1].join("/")
        + "/"
        + &api_name_str
        + ".py";
    println!("script_path: {:?}", script_path);

    let output = std::process::Command::new(GLOBAL_CONFIG.read().unwrap().python_path.clone())
        .arg(script_path)
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
