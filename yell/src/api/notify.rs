use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use share_lib::err_mapping::MailManErrResponser;
use std::collections::HashMap;

use crate::{
    api::filter, services::notification_router::NotificationRouter, services::notify_service,
};

// ==================== 统一通知发送 API ====================

/// POST /api/notify/template - 使用模板发送
pub async fn send_with_template(
    req: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let input = filter::SendTemplateRequestInput::from_map(&req);

    let template_name = input.template_name.as_deref().ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'template_name' field".to_string()),
            1,
        ))
    })?;

    let variables = input.variables;

    let recipients = match notify_service::resolve_recipients(req.get("recipients"), &pool).await {
        Ok(recipients) => recipients,
        Err(err) => return Err(MailManErrResponser::mapping_from_mme(err)),
    };

    let template = match notify_service::get_template_by_name(template_name, &pool).await {
        Ok(template) => template,
        Err(err) => return Err(MailManErrResponser::mapping_from_mme(err)),
    };

    // recipients 为空时 router 不产生发送任务，保持不查渠道配置的原行为
    let channel_configs = if recipients.is_empty() {
        HashMap::new()
    } else {
        match notify_service::load_channel_configs(&pool).await {
            Ok(configs) => configs,
            Err(err) => return Err(MailManErrResponser::mapping_from_mme(err)),
        }
    };

    let router = NotificationRouter::new();

    match router
        .send_with_template(template, variables, recipients, channel_configs, &pool)
        .await
    {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}
