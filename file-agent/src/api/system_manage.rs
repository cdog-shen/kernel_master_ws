use actix_web::HttpResponse;

use serde_json::json;
use share_lib::{
    data_structure::MailManOk, err_mapping::MailManErrResponser, infrastructure::master_registry,
};

use crate::server;

// send a request to master to refresh the uuid
pub async fn refresh_master() -> Result<HttpResponse, MailManErrResponser> {
    let (base_url, register_name, subsys_uuid) = {
        let config = server::GLOBAL_CONFIG.read().unwrap();
        (
            format!("http://{}:{}/api", config.master_addr, config.master_port),
            config.register_name.clone(),
            config.subsys_uuid.clone(),
        )
    };

    match master_registry::refresh_master_registration(&base_url, &register_name, &subsys_uuid) {
        Ok(res_data) => {
            Ok(HttpResponse::Ok().json(MailManOk::new(200, "Master refreshed", Some(res_data))))
        }
        Err(mme_obj) => Err(MailManErrResponser::mapping_from_mme(mme_obj)),
    }
}

// 运维部署辅助：按 watchman 子系统注册接口（POST /api/subsystem_control/new_subsystem）
// 所需字段预填一份注册 JSON，供运维直接拿去注册
// 安全要求：响应携带 subsys_uuid（子系统凭证），必须过 UserAuth 鉴权，不得加入 authenticate_bypass
pub async fn register_help() -> Result<HttpResponse, MailManErrResponser> {
    let (register_name, listen_addr, listen_port, subsys_uuid) = {
        let config = server::GLOBAL_CONFIG.read().unwrap();
        (
            config.register_name.clone(),
            config.listen_addr.clone(),
            config.listen_port,
            config.subsys_uuid.clone(),
        )
    };

    // listen_addr 是本机监听地址（可能为 0.0.0.0 这类通配地址），
    // 运维注册前需将 url 替换为 watchman 实际可达的地址
    let url = format!("http://{listen_addr}:{listen_port}");

    // 字段集对齐 watchman SubsysInfo::from_map / new_subsys：
    // subsys_name / url / token 必填，relate_service_id 默认 0，is_enable 预置 true
    let payload = json!({
        "subsys_name": register_name,
        "url": url,
        "token": subsys_uuid,
        "relate_service_id": 0,
        "is_enable": true,
    });

    Ok(HttpResponse::Ok().json(MailManOk::new(200, "register help payload", Some(payload))))
}
