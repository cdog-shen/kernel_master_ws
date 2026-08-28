use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use share_lib::err_mapping::MailManErrResponser;

use crate::{
    api::filter,
    model::notification_template::{NewNotificationTemplate, UpdateNotificationTemplate},
    services::manage_service,
};

// ==================== 模板管理 API ====================

/// GET /api/template/get - 获取模板列表
pub async fn get_templates(
    query: web::Query<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match manage_service::get_templates(filter::clean_template_filter(query.into_inner()), &pool)
        .await
    {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

/// GET /api/template/help - 获取指定模板的模拟渲染示例（query 传 name）
pub async fn get_template_help(
    query: web::Query<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    template_help(query.get("name"), &pool).await
}

/// POST /api/template/help - 同上（body 传 name）
pub async fn post_template_help(
    req: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    template_help(req.get("name"), &pool).await
}

/// 清洗 name 参数（缺失或非字符串返回 400）并调用编排层获取模拟渲染示例
async fn template_help(
    name: Option<&Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let name = name.and_then(|v| v.as_str()).ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'name' field".to_string()),
            1,
        ))
    })?;

    match manage_service::get_template_example(name, pool).await {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

/// POST /api/template/new - 创建模板
pub async fn create_template(
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

    let new_template = NewNotificationTemplate {
        name: name.to_string(),
        description: req
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        is_enabled: req.get("is_enabled").and_then(|v| v.as_bool()),
        smtp: req.get("smtp").cloned(),
        bark: req.get("bark").cloned(),
        gotify: req.get("gotify").cloned(),
        teams_hook: req.get("teams_hook").cloned(),
        webhook: req.get("webhook").cloned(),
    };

    match manage_service::create_template(new_template, &pool).await {
        Ok(id) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Template created",
                Some(format!("Template ID: {}", id)),
            )),
        ),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

/// POST /api/template/update - 更新模板
pub async fn update_template(
    req: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let template_id = req.get("id").and_then(|v| v.as_i64()).ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'id' field".to_string()),
            1,
        ))
    })? as i32;

    let update = UpdateNotificationTemplate {
        name: req
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        description: req
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        is_enabled: req.get("is_enabled").and_then(|v| v.as_bool()),
        smtp: req.get("smtp").cloned(),
        bark: req.get("bark").cloned(),
        gotify: req.get("gotify").cloned(),
        teams_hook: req.get("teams_hook").cloned(),
        webhook: req.get("webhook").cloned(),
        updated_at: Some(chrono::Local::now().naive_local()),
    };

    match manage_service::update_template(template_id, update, &pool).await {
        Ok(num) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Template updated",
                Some(format!("Rows affected: {}", num)),
            )),
        ),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}

/// POST /api/template/delete - 删除模板
pub async fn delete_template(
    req: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let template_id = req.get("id").and_then(|v| v.as_i64()).ok_or_else(|| {
        MailManErrResponser::mapping_from_mme(share_lib::data_structure::MailManErr::new(
            400,
            "Bad Request",
            Some("Missing 'id' field".to_string()),
            1,
        ))
    })? as i32;

    match manage_service::delete_template(template_id, &pool).await {
        Ok(num) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Template deleted",
                Some(format!("Rows affected: {}", num)),
            )),
        ),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}
