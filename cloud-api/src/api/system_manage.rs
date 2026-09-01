use actix_web::HttpResponse;

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

// 按 watchman `POST /api/subsystem_control/new_subsystem` 所需字段预填注册 JSON，
// 供运维部署时直接取用；含 subsys_uuid（子系统凭证），必须过 UserAuth 鉴权
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

    // url 由监听地址组装；若 listen_addr 为 0.0.0.0/:: 这类通配监听地址，
    // 运维注册前需替换为 watchman 实际可达的地址
    let register_info = serde_json::json!({
        "subsys_name": register_name,
        "url": format!("http://{listen_addr}:{listen_port}"),
        "token": subsys_uuid,
        "relate_service_id": 0,
        "is_enable": true,
    });

    Ok(HttpResponse::Ok().json(MailManOk::new(
        200,
        "Register info generated",
        Some(register_info),
    )))
}
