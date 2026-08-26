use serde::Deserialize;
use serde_json::Value;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::config::worker;
use crate::service::json_rpc::update_log;

/// MQ 任务消息（清洗层强类型输入）
///
/// 字段与 jc-commander 投递侧（`service/script_caller.rs`）拼的 payload 兼容：
/// `id` / `commander` 由 commander 注入，`script` / `params` 来自调用方请求。
#[derive(Debug, Deserialize)]
pub struct TaskPayload {
    /// 任务 uuid（commander 注入）
    pub id: String,
    /// 调用方 commander 子系统 uuid（commander 注入）
    pub commander: String,
    /// 脚本名（按下划线分段映射脚本目录）
    pub script: String,
    /// 传给脚本的参数（原样 JSON 序列化后作为 argv 传入；缺省为 null）
    #[serde(default)]
    pub params: Value,
}

// async task exe
pub async fn execute<'a>(payload: &[u8]) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    // payload 清洗：反序列化 + 必填字段校验。
    // 失败视为毒消息：记 error 日志并返回 Err，由消费循环 ack 丢弃，
    // 不 panic、不 redelivery，不影响消费循环存活。
    let payload = match serde_json::from_slice::<TaskPayload>(payload) {
        Ok(p) if !p.id.is_empty() && !p.commander.is_empty() && !p.script.is_empty() => p,
        Ok(p) => {
            return Err(MailManErr::new(
                400,
                "task payload missing required field",
                Some(format!("{p:?}")),
                1,
            ));
        }
        Err(e) => {
            return Err(MailManErr::new(
                400,
                "task payload deserialize failed",
                Some(e.to_string()),
                1,
            ));
        }
    };

    log::info!("Here comes Payload: {payload:?}");

    let auth = payload.commander.clone();
    let uuid = payload.id.clone();
    let script_name = payload.script.clone();
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
            .arg(payload.params.to_string())
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
                    payload.params,
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
                    payload.params,
                    err_msg
                );
                Ok(err_msg)
            } else {
                log::error!("Task {script_name} with params {auth} Nothing in stdout",);
                Err("Nothing in stdout".to_string())
            }
        }
        Err(ref e) => Err(format!("Error: {e}")),
    };

    let (result, status) = match &output {
        Ok(o) if o.status.success() => (res.clone().unwrap_or_else(|e| e), 2),
        _ => {
            MailManErr::new(500, "task execute Error", Some(res.clone()), 1);
            (
                res.err().unwrap_or_else(|| "Execution failed".to_string()),
                0,
            )
        }
    };

    match update_log(auth, uuid.to_string(), status, result).await {
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
