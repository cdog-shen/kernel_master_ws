use chrono::Local;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::config::worker;

pub fn update_log<'a>(
    auth: String,
    uuid: String,
    status: u8,
    result: String,
) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    let commander_url = format!(
        "http://{}:{}/api/log/update",
        worker::GLOBAL_CONFIG.read().unwrap().commander_addr,
        worker::GLOBAL_CONFIG.read().unwrap().commander_port
    );
    let worker_id = &worker::GLOBAL_CONFIG.read().unwrap().subsys_uuid;
    let response = ureq::post(commander_url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("uuid {}", auth))
        .send(
            serde_json::to_string(&serde_json::json!({
                "id": uuid,
                "worker": worker_id,
                "status": status,
                "result": result,
                "finish_time": Local::now().naive_local().format("%Y-%m-%dT%H:%M:%S").to_string(),
            }))
            .unwrap(),
        );

    match response {
        Ok(resp) => {
            if resp.status() == 200 {
                return Ok(MailManOk::new(
                    200,
                    "log update Done",
                    Some(resp.into_body().read_to_string().unwrap()),
                ));
            } else {
                return Err(MailManErr::new(
                    500,
                    "log update Failed",
                    resp.into_body().read_to_string().unwrap(),
                    1,
                ));
            }
        }
        Err(e) => return Err(MailManErr::new(500, "log update Failed", e, 1)),
    }
}
