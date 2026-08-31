use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use share_lib::err_mapping::MailManErrResponser;

use crate::{
    model::notification_alias::{NewNotificationAlias, UpdateNotificationAlias},
    services::{manage_service, notify_service},
};

// ==================== Alias 管理 API ====================

/// GET /api/alias/get - 获取所有 alias
pub async fn get_aliases(
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match manage_service::get_aliases(&pool).await {
        Ok(data) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Aliases fetched",
                Some(data),
            )),
        ),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

/// POST /api/alias/new - 创建 alias
pub async fn create_alias(
    req: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let name = req.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'name' field".to_string()),
            1,
        ))
    })?;

    let recipients = req.get("recipients").cloned().ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'recipients' field".to_string()),
            1,
        ))
    })?;

    if let Err(msg) = notify_service::validate_recipients_shape(&recipients) {
        return Err(MailManErrResponser::mapping_from_mme(
            share_lib::data_structure::MailManErr::new(400, "Bad Request", Some(msg), 1),
        ));
    }

    let new_alias = NewNotificationAlias {
        name: name.to_string(),
        description: req
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        recipients,
        is_enabled: req.get("is_enabled").and_then(|v| v.as_bool()),
    };

    match manage_service::create_alias(new_alias, &pool).await {
        Ok(id) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Alias created",
                Some(format!("Alias ID: {}", id)),
            )),
        ),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

/// POST /api/alias/update - 更新 alias
pub async fn update_alias(
    req: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let alias_id = req.get("id").and_then(|v| v.as_i64()).ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'id' field".to_string()),
            1,
        ))
    })? as i32;

    if let Some(recipients) = req.get("recipients") {
        if let Err(msg) = notify_service::validate_recipients_shape(recipients) {
            return Err(MailManErrResponser::mapping_from_mme(
                share_lib::data_structure::MailManErr::new(400, "Bad Request", Some(msg), 1),
            ));
        }
    }

    let update = UpdateNotificationAlias {
        name: req
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        description: req
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        recipients: req.get("recipients").cloned(),
        is_enabled: req.get("is_enabled").and_then(|v| v.as_bool()),
        updated_at: Some(chrono::Local::now().naive_local()),
    };

    match manage_service::update_alias(alias_id, update, &pool).await {
        Ok(num) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Alias updated",
                Some(format!("Rows affected: {}", num)),
            )),
        ),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

/// POST /api/alias/delete - 删除 alias
pub async fn delete_alias(
    req: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let alias_id = req.get("id").and_then(|v| v.as_i64()).ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'id' field".to_string()),
            1,
        ))
    })? as i32;

    match manage_service::delete_alias(alias_id, &pool).await {
        Ok(num) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Alias deleted",
                Some(format!("Rows affected: {}", num)),
            )),
        ),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}
