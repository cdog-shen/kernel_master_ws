use serde_json::Value;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::config::worker;
use crate::service::json_rpc::update_log;

// sync task exe
pub async fn execute<'a>(payload: &[u8]) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    let payload =
        serde_json::from_str::<Value>(&String::from_utf8(payload.to_vec()).unwrap()).unwrap();

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
    script_path.push_str(&format!("{}.py", script_name));

    let output =
        std::process::Command::new(worker::GLOBAL_CONFIG.read().unwrap().python_path.clone())
            .arg(script_path)
            .arg(payload["params"].to_string())
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

    let (result, status) = match res {
        Ok(res_data) => (res_data, 2),
        Err(e) => {
            MailManErr::new(500, "tast execute Error", e.clone(), 1);
            (e, 0)
        }
    };

    let auth = payload.get("commander").unwrap();

    match update_log(auth.to_string(), uuid.to_string(), status, result) {
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
