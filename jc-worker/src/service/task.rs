use serde_json::Value;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::config::worker;
use crate::service::json_rpc::update_log;

// async task exe
pub async fn execute<'a>(payload: &[u8]) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    let payload =
        serde_json::from_str::<Value>(core::str::from_utf8(payload).expect("Decode Error"))
            .expect("Deserialize Error");

    log::info!("Here comes Payload: {payload}");

    let auth = payload
        .get("commander")
        .unwrap()
        .as_str()
        .unwrap()
        .trim_matches('"')
        .to_string();
    let uuid = payload
        .get("id")
        .unwrap()
        .as_str()
        .unwrap()
        .trim_matches('"')
        .to_string();
    let script_name = payload
        .get("script")
        .unwrap()
        .as_str()
        .unwrap()
        .trim_matches('"')
        .to_string();
    let mut script_path = worker::GLOBAL_CONFIG.read().unwrap().script_dir.clone() + "/";

    for name in script_name.split('_') {
        if name != script_name.split("_").last().unwrap() {
            script_path.push_str(name);
            script_path.push('/');
        }
    }
    script_path.push_str(&format!("{script_name}.py"));

    let output =
        std::process::Command::new(worker::GLOBAL_CONFIG.read().unwrap().python_path.clone())
            .arg(script_path)
            .arg(payload["params"].to_string())
            .output();

    let res = match output {
        Ok(ref output) => {
            if !output.stdout.is_empty() {
                let res_data =
                    String::from_utf8(output.stdout.clone()).expect("Error: stdout is not utf8");
                // log info
                log::info!(
                    "Task {} with params {} Info: {}",
                    script_name,
                    payload["params"],
                    res_data
                );
                Ok(res_data)
            } else if !output.stderr.is_empty() {
                let err_msg =
                    String::from_utf8(output.stderr.clone()).expect("Error: stderr is not utf8");
                // log error
                log::error!(
                    "Task {} with params {} Error: {}",
                    script_name,
                    payload["params"],
                    err_msg
                );
                Ok(err_msg)
            } else {
                log::error!(
                    "Task {script_name} with params {auth} Nothing in stdout",
                );
                Err("Nothing in stdout".to_string())
            }
        }
        Err(ref e) => Err(format!("Error: {e}")),
    };

    let (result, status) = match output.unwrap().status {
        code if code.success() => (res.unwrap(), 2),
        _ => {
            MailManErr::new(500, "task execute Error", Some(res.clone()), 1);
            (
                res.err().unwrap_or_else(|| "Execution failed".to_string()),
                0,
            )
        }
    };

    match update_log(auth, uuid.to_string(), status, result) {
        Ok(MailManOk {
            code: _,
            key: _,
            data: resu,
        }) => Ok(MailManOk::new(200, "task Finish with OK", resu)),
        Err(MailManErr {
            code: _,
            key: _,
            msg: e,
            level: _,
        }) => Err(MailManErr::new(500, "task Finish with Error", e, 1)),
    }
}
