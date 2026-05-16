use chrono::{self, Local};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::notification_templates::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = notification_templates)]
pub struct NotificationTemplateModel {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub channel_type: String,
    pub subject_template: Option<String>,
    pub content_template: String,
    pub content_format: Option<String>,
    pub is_enabled: Option<bool>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = notification_templates)]
pub struct NotificationTemplateInfo {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub channel_type: Option<String>,
    pub subject_template: Option<String>,
    pub content_template: Option<String>,
    pub content_format: Option<String>,
    pub is_enabled: Option<bool>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

impl NotificationTemplateInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(NotificationTemplateInfo {
            id: match map.get("id") {
                Some(value) => value.as_i64().map(|v| v as i32),
                None => None,
            },
            name: match map.get("name") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            description: match map.get("description") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            channel_type: match map.get("channel_type") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            subject_template: match map.get("subject_template") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            content_template: match map.get("content_template") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            content_format: match map.get("content_format") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            is_enabled: match map.get("is_enabled") {
                Some(value) => value.as_bool(),
                None => None,
            },
            created_at: match map.get("created_at") {
                Some(_) => None,
                None => None,
            },
            updated_at: Some(Local::now().naive_local()),
        })
    }
}

impl NotificationTemplateModel {
    /// get model full data
    pub fn get_model_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = notification_templates
            .into_boxed()
            .select(NotificationTemplateModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "channel_type" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(channel_type.eq(value));
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

        match query.get_results::<NotificationTemplateModel>(conn) {
            Ok(vec_item_info) => {
                let vec_value: Vec<Value> = vec_item_info
                    .into_iter()
                    .map(|item| serde_json::to_value(item).unwrap_or(Value::Null))
                    .collect();
                Ok(vec_value)
            }
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}

impl NotificationTemplateModel {
    pub fn new(
        info: &NotificationTemplateInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::insert_into(notification_templates)
            .values(info)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(
        info: &NotificationTemplateInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(notification_templates.filter(id.eq(info.id.unwrap())))
            .set(info)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((
                    BAD_REQUEST_CODE,
                    format!("id: {} not found", info.id.unwrap()),
                )),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete(_id: i32, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(notification_templates.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {} not found", _id))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
