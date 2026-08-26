use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::notification_templates::{self, dsl::*};

static UNKNOWN_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = notification_templates, check_for_backend(diesel::pg::Pg))]
pub struct NotificationTemplate {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub is_enabled: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub params_template: Option<Value>,
    pub smtp: Option<Value>,
    pub bark: Option<Value>,
    pub gotify: Option<Value>,
    pub ntfy: Option<Value>,
    pub teams: Option<Value>,
    pub webhook: Option<Value>,
}

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = notification_templates)]
pub struct NewNotificationTemplate {
    pub name: String,
    pub description: Option<String>,
    pub is_enabled: Option<bool>,
    pub params_template: Option<Value>,
    pub smtp: Option<Value>,
    pub bark: Option<Value>,
    pub gotify: Option<Value>,
    pub ntfy: Option<Value>,
    pub teams: Option<Value>,
    pub webhook: Option<Value>,
}

#[derive(AsChangeset, Debug, Serialize, Deserialize)]
#[diesel(table_name = notification_templates)]
pub struct UpdateNotificationTemplate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_enabled: Option<bool>,
    pub params_template: Option<Value>,
    pub smtp: Option<Value>,
    pub bark: Option<Value>,
    pub gotify: Option<Value>,
    pub ntfy: Option<Value>,
    pub teams: Option<Value>,
    pub webhook: Option<Value>,
    pub updated_at: Option<NaiveDateTime>,
}

impl NotificationTemplate {
    /// 根据 ID 获取模板
    pub fn get_by_id(
        template_id: i32,
        conn: &mut PgConnection,
    ) -> Result<Option<Self>, (u8, String)> {
        match notification_templates
            .filter(id.eq(template_id))
            .filter(is_enabled.eq(Some(true)))
            .select(NotificationTemplate::as_select())
            .first(conn)
        {
            Ok(template) => Ok(Some(template)),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 根据名称获取模板
    pub fn get_by_name(
        template_name: &str,
        conn: &mut PgConnection,
    ) -> Result<Option<Self>, (u8, String)> {
        match notification_templates
            .filter(name.eq(template_name))
            .filter(is_enabled.eq(Some(true)))
            .select(NotificationTemplate::as_select())
            .first(conn)
        {
            Ok(template) => Ok(Some(template)),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 根据过滤器获取模板列表
    pub fn get_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = notification_templates
            .into_boxed()
            .select(NotificationTemplate::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "is_enabled" => {
                    if let Some(value) = q_v.as_bool() {
                        query = query.filter(is_enabled.eq(value));
                    }
                }
                "name" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(name.eq(value));
                    }
                }
                _ => continue,
            }
        }

        match query.load::<NotificationTemplate>(conn) {
            Ok(templates) => Ok(templates
                .into_iter()
                .map(|t| serde_json::to_value(&t).unwrap())
                .collect()),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 创建新模板
    pub fn create(
        new_template: &NewNotificationTemplate,
        conn: &mut PgConnection,
    ) -> Result<i32, (u8, String)> {
        match diesel::insert_into(notification_templates)
            .values(new_template)
            .returning(id)
            .get_result::<i32>(conn)
        {
            Ok(new_id) => Ok(new_id),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 更新模板
    pub fn update(
        template_id: i32,
        update: &UpdateNotificationTemplate,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(notification_templates.filter(id.eq(template_id)))
            .set(update)
            .execute(conn)
        {
            Ok(num) => match num {
                0 => Err((
                    BAD_REQUEST_CODE,
                    format!("Template id: {} not found", template_id),
                )),
                _ => Ok(num),
            },
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 删除模板
    pub fn delete(template_id: i32, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(notification_templates.filter(id.eq(template_id))).execute(conn) {
            Ok(num) => match num {
                0 => Err((
                    BAD_REQUEST_CODE,
                    format!("Template id: {} not found", template_id),
                )),
                _ => Ok(num),
            },
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }
}
