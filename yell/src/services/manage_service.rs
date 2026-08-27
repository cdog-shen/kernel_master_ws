//! 管理端编排层 —— template / record / channel / alias 的 CRUD 编排
//!
//! 清洗层（api/notify.rs）只负责参数解析与校验，
//! 本模块负责组织 diesel 同步调用（web::block 包裹）并将结果映射为 MailMan 体系。

use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use share_lib::data_structure::MailManErr;

use crate::model::{
    channel_config::ChannelConfig,
    notification_alias::{NewNotificationAlias, NotificationAlias, UpdateNotificationAlias},
    notification_record::NotificationRecord,
    notification_template::{
        NewNotificationTemplate, NotificationTemplate, UpdateNotificationTemplate,
    },
};

// ==================== 模板管理 ====================

/// 获取模板列表
pub async fn get_templates<'a>(
    filter: Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<Vec<Value>, MailManErr<'a, String>> {
    let result = web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().map_err(|e| (0, e.to_string()))?;
            NotificationTemplate::get_with_filter(&filter, &mut conn)
        }
    })
    .await;

    match result {
        Ok(Ok(data)) => Ok(data),
        Ok(Err((_, msg))) => Err(MailManErr::new(
            500,
            "Failed to get templates",
            Some(msg),
            0,
        )),
        Err(e) => Err(MailManErr::new(
            500,
            "Failed to get templates",
            Some(e.to_string()),
            0,
        )),
    }
}

/// 创建模板，返回新模板 ID
pub async fn create_template<'a>(
    new_template: NewNotificationTemplate,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<i32, MailManErr<'a, String>> {
    let result = web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().map_err(|e| (0, e.to_string()))?;
            NotificationTemplate::create(&new_template, &mut conn)
        }
    })
    .await;

    match result {
        Ok(Ok(id)) => Ok(id),
        Ok(Err((_, msg))) => Err(MailManErr::new(
            500,
            "Failed to create template",
            Some(msg),
            0,
        )),
        Err(e) => Err(MailManErr::new(
            500,
            "Failed to create template",
            Some(e.to_string()),
            0,
        )),
    }
}

/// 更新模板，返回受影响行数
pub async fn update_template<'a>(
    template_id: i32,
    update: UpdateNotificationTemplate,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<usize, MailManErr<'a, String>> {
    let result = web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().map_err(|e| (0, e.to_string()))?;
            NotificationTemplate::update(template_id, &update, &mut conn)
        }
    })
    .await;

    match result {
        Ok(Ok(num)) => Ok(num),
        Ok(Err((_, msg))) => Err(MailManErr::new(
            500,
            "Failed to update template",
            Some(msg),
            0,
        )),
        Err(e) => Err(MailManErr::new(
            500,
            "Failed to update template",
            Some(e.to_string()),
            0,
        )),
    }
}

/// 删除模板，返回受影响行数
pub async fn delete_template<'a>(
    template_id: i32,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<usize, MailManErr<'a, String>> {
    let result = web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().map_err(|e| (0, e.to_string()))?;
            NotificationTemplate::delete(template_id, &mut conn)
        }
    })
    .await;

    match result {
        Ok(Ok(num)) => Ok(num),
        Ok(Err((_, msg))) => Err(MailManErr::new(
            500,
            "Failed to delete template",
            Some(msg),
            0,
        )),
        Err(e) => Err(MailManErr::new(
            500,
            "Failed to delete template",
            Some(e.to_string()),
            0,
        )),
    }
}

// ==================== 通知记录查询 ====================

/// 获取通知记录
pub async fn get_records<'a>(
    filter: Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<Vec<Value>, MailManErr<'a, String>> {
    let result = web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().map_err(|e| (0, e.to_string()))?;
            NotificationRecord::get_with_filter(&filter, &mut conn)
        }
    })
    .await;

    match result {
        Ok(Ok(data)) => Ok(data),
        Ok(Err((_, msg))) => Err(MailManErr::new(500, "Failed to get records", Some(msg), 0)),
        Err(e) => Err(MailManErr::new(
            500,
            "Failed to get records",
            Some(e.to_string()),
            0,
        )),
    }
}

// ==================== 通道配置 ====================

/// 获取全部通道配置
pub async fn get_channel_configs<'a>(
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<Vec<Value>, MailManErr<'a, String>> {
    let result = web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().map_err(|e| (0, e.to_string()))?;
            ChannelConfig::get_all(&mut conn)
        }
    })
    .await;

    match result {
        Ok(Ok(data)) => Ok(data),
        Ok(Err((_, msg))) => Err(MailManErr::new(
            500,
            "Failed to get channel configs",
            Some(msg),
            0,
        )),
        Err(e) => Err(MailManErr::new(
            500,
            "Failed to get channel configs",
            Some(e.to_string()),
            0,
        )),
    }
}

/// 更新（upsert）通道配置，返回受影响行数
pub async fn update_channel_config<'a>(
    channel_type: String,
    instance_name: String,
    config_json: Value,
    is_enabled: bool,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<usize, MailManErr<'a, String>> {
    let result = web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().map_err(|e| (0, e.to_string()))?;
            ChannelConfig::upsert(
                &channel_type,
                &instance_name,
                &config_json,
                is_enabled,
                &mut conn,
            )
        }
    })
    .await;

    match result {
        Ok(Ok(num)) => Ok(num),
        Ok(Err((_, msg))) => Err(MailManErr::new(
            500,
            "Failed to update channel config",
            Some(msg),
            0,
        )),
        Err(e) => Err(MailManErr::new(
            500,
            "Failed to update channel config",
            Some(e.to_string()),
            0,
        )),
    }
}

// ==================== Alias 管理 ====================

/// 获取全部 alias
pub async fn get_aliases<'a>(
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<Vec<Value>, MailManErr<'a, String>> {
    let result = web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().map_err(|e| (0, e.to_string()))?;
            NotificationAlias::get_all(&mut conn)
        }
    })
    .await;

    match result {
        Ok(Ok(data)) => Ok(data),
        Ok(Err((_, msg))) => Err(MailManErr::new(500, "Failed to get aliases", Some(msg), 0)),
        Err(e) => Err(MailManErr::new(
            500,
            "Failed to get aliases",
            Some(e.to_string()),
            0,
        )),
    }
}

/// 创建 alias，返回新 alias ID
pub async fn create_alias<'a>(
    new_alias: NewNotificationAlias,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<i32, MailManErr<'a, String>> {
    let result = web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().map_err(|e| (0, e.to_string()))?;
            NotificationAlias::create(&new_alias, &mut conn)
        }
    })
    .await;

    match result {
        Ok(Ok(id)) => Ok(id),
        Ok(Err((_, msg))) => Err(MailManErr::new(500, "Failed to create alias", Some(msg), 0)),
        Err(e) => Err(MailManErr::new(
            500,
            "Failed to create alias",
            Some(e.to_string()),
            0,
        )),
    }
}

/// 更新 alias，返回受影响行数
pub async fn update_alias<'a>(
    alias_id: i32,
    update: UpdateNotificationAlias,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<usize, MailManErr<'a, String>> {
    let result = web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().map_err(|e| (0, e.to_string()))?;
            NotificationAlias::update(alias_id, &update, &mut conn)
        }
    })
    .await;

    match result {
        Ok(Ok(num)) => Ok(num),
        Ok(Err((_, msg))) => Err(MailManErr::new(500, "Failed to update alias", Some(msg), 0)),
        Err(e) => Err(MailManErr::new(
            500,
            "Failed to update alias",
            Some(e.to_string()),
            0,
        )),
    }
}

/// 删除 alias，返回受影响行数
pub async fn delete_alias<'a>(
    alias_id: i32,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<usize, MailManErr<'a, String>> {
    let result = web::block({
        let pool = pool.clone();
        move || {
            let mut conn = pool.get().map_err(|e| (0, e.to_string()))?;
            NotificationAlias::delete(alias_id, &mut conn)
        }
    })
    .await;

    match result {
        Ok(Ok(num)) => Ok(num),
        Ok(Err((_, msg))) => Err(MailManErr::new(500, "Failed to delete alias", Some(msg), 0)),
        Err(e) => Err(MailManErr::new(
            500,
            "Failed to delete alias",
            Some(e.to_string()),
            0,
        )),
    }
}
