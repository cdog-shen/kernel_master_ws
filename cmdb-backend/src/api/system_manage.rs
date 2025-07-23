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

    let id_res = ureq::get(&format!("{base_url}/subsystem_control/all_subsystem"))
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

    let req_json = serde_json::json!({
        "id": this_id,
        "subsys_name": server::GLOBAL_CONFIG.read().unwrap().register_name,
        "token" : server::GLOBAL_CONFIG.read().unwrap().subsys_uuid,
    });

    let res = ureq::post(&format!("{base_url}/subsystem_control/update_subsystem"))
        .header("Connection", "close")
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
