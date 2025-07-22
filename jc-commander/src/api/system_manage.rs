use actix_web::HttpResponse;
use ureq;

use share_lib::data_structure::MailManOk;

use crate::server;

// send a request to master to refresh the uuid
pub async fn refresh_master() -> HttpResponse {
    let config = server::GLOBAL_CONFIG.read().unwrap();
    let base_url = format!("http://{}:{}/api", config.master_addr, config.master_port);
    let register_name = config.register_name.clone();
    drop(config); // Drop the MutexGuard here

    let id_res = ureq::get(&format!("{}/subsystem_control/all_subsystem", base_url))
        .query("subsys_name", &register_name)
        // .timeout(std::time::Duration::from_millis(1000))
        .call();

    let this_id = match id_res {
        Ok(res) => {
            println!("res status: {}", res.status());
            if res.status() == 200 {
                let json_value: serde_json::Value =
                    serde_json::from_reader(res.into_body().into_reader()).unwrap();
                json_value["data"][0]["id"].clone()
            } else {
                return HttpResponse::InternalServerError().json("Unexpected response status");
            }
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(err.to_string());
        }
    };

    let config = server::GLOBAL_CONFIG.read().unwrap();
    let req_json = serde_json::json!({
        "id": this_id,
        "subsys_name": config.register_name.clone(),
        "token" : config.subsys_uuid.clone(),
    });
    drop(config); // Drop the MutexGuard here

    let res = ureq::post(&format!("{}/subsystem_control/update_subsystem", base_url))
        .header("Content-Type", "application/json")
        .send(serde_json::to_string(&req_json).unwrap());

    match res {
        Ok(res_data) => HttpResponse::Ok().json(MailManOk::new(
            200,
            "Master refreshed",
            Some(res_data.into_body().read_to_string().unwrap()),
        )),
        Err(err_data) => HttpResponse::InternalServerError().json(err_data.to_string()),
    }
}
