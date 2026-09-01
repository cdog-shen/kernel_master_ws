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

// 部署运维用：返回按 watchman POST /api/subsystem_control/new_subsystem 所需字段
// 预填好的注册请求体 JSON，可直接拿去注册本子系统
// 注意：响应含 subsys_uuid（token 字段），必须过 UserAuth 鉴权，不得进 authenticate_bypass
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

    // listen_addr 是监听地址（可能为 0.0.0.0），watchman 侧需要实际可达地址，
    // 运维注册前需把 url 替换为本机对外可达地址
    let register_body = serde_json::json!({
        "subsys_name": register_name,
        "url": format!("http://{listen_addr}:{listen_port}"),
        "token": subsys_uuid,
        "is_enable": true,
        "relate_service_id": 0,
    });

    Ok(HttpResponse::Ok().json(MailManOk::new(
        200,
        "Register help generated",
        Some(register_body),
    )))
}
