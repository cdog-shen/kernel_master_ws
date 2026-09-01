use actix_web::HttpResponse;

use share_lib::{
    data_structure::MailManOk, err_mapping::MailManErrResponser, infrastructure::master_registry,
};

use crate::server;

// 向 master 发送请求以刷新本子系统的 uuid 注册信息
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

// 返回按 watchman 子系统注册接口（POST /api/subsystem_control/new_subsystem）字段
// 预填好的注册 JSON，供运维部署时直接拿去注册本子系统
pub async fn register_help() -> Result<HttpResponse, MailManErrResponser> {
    let (register_name, url, subsys_uuid) = {
        let config = server::GLOBAL_CONFIG.read().unwrap();
        (
            config.register_name.clone(),
            // listen_addr 是监听地址（常为 0.0.0.0），运维注册时需替换为实际可达地址
            format!("http://{}:{}", config.listen_addr, config.listen_port),
            config.subsys_uuid.clone(),
        )
    };

    Ok(HttpResponse::Ok().json(MailManOk::new(
        200,
        "Register help generated",
        Some(serde_json::json!({
            "subsys_name": register_name,
            "url": url,
            "token": subsys_uuid,
            "is_enable": true,
            "relate_service_id": 0,
        })),
    )))
}
