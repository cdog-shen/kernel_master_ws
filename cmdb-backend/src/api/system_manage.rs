use actix_web::HttpResponse;
use ureq;

use share_lib::data_structure::MailManOk;

use crate::server;

// send a request to master to refresh the uuid
pub async fn refresh_master() -> HttpResponse {
    let base_url = format!(
        "http://{}:{}/api",
        server::GLOBAL_CONFIG.read().unwrap().master_addr,
        server::GLOBAL_CONFIG.read().unwrap().master_port
    );

    let id_res = ureq::get(&format!("{}/subsystem_control/all_subsystem", base_url))
        .query(
            "subsys_name",
            &server::GLOBAL_CONFIG.read().unwrap().register_name,
        )
        // .timeout(std::time::Duration::from_millis(1000))
        .call();

    let this_id = match id_res {
        Ok(res) => {
            println!("res status: {}", res.status());
            if res.status() == 200 {
                let json_value: serde_json::Value = res.into_json().unwrap();
                json_value["data"][0]["id"].clone()
            } else {
                return HttpResponse::InternalServerError().json("Unexpected response status");
            }
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(err.to_string());
        }
    };

    let req_json = serde_json::json!({
        "id": this_id,
        "subsys_name": server::GLOBAL_CONFIG.read().unwrap().register_name,
        "token" : server::GLOBAL_CONFIG.read().unwrap().subsys_uuid,
    });

    let res: Result<serde_json::Value, std::io::Error> =
        ureq::post(&format!("{}/subsystem_control/update_subsystem", base_url))
            .send_json(req_json)
            .unwrap()
            .into_json();

    match res {
        Ok(res_data) => {
            HttpResponse::Ok().json(MailManOk::new(200, "Master refreshed", Some(res_data)))
        }
        Err(err_data) => HttpResponse::InternalServerError().json(err_data.to_string()),
    }
}
