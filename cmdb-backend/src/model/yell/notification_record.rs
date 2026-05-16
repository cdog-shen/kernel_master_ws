use chrono::{self};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::notification_records::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = notification_records)]
pub struct NotificationRecordModel {
    pub id: i32,
    pub channel_type: String,
    pub recipient: String,
    pub subject: Option<String>,
    pub content: String,
    pub content_format: Option<String>,
    pub template_id: Option<i32>,
    pub status: Option<String>,
    pub error_msg: Option<String>,
    pub sent_at: Option<chrono::NaiveDateTime>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = notification_records)]
pub struct NotificationRecordInfo {
    pub id: Option<i32>,
    pub channel_type: Option<String>,
    pub recipient: Option<String>,
    pub subject: Option<String>,
    pub content: Option<String>,
    pub content_format: Option<String>,
    pub template_id: Option<i32>,
    pub status: Option<String>,
    pub error_msg: Option<String>,
    pub sent_at: Option<chrono::NaiveDateTime>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

impl NotificationRecordInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(NotificationRecordInfo {
            id: match map.get("id") {
                Some(value) => value.as_i64().map(|v| v as i32),
                None => None,
            },
            channel_type: match map.get("channel_type") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            recipient: match map.get("recipient") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            subject: match map.get("subject") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            content: match map.get("content") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            content_format: match map.get("content_format") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            template_id: match map.get("template_id") {
                Some(value) => value.as_i64().map(|v| v as i32),
                None => None,
            },
            status: match map.get("status") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            error_msg: match map.get("error_msg") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            sent_at: match map.get("sent_at") {
                Some(_) => None,
                None => None,
            },
            created_at: match map.get("created_at") {
                Some(_) => None,
                None => None,
            },
            updated_at: match map.get("updated_at") {
                Some(_) => None,
                None => None,
            },
        })
    }
}

impl NotificationRecordModel {
    /// get model full data
    pub fn get_model_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = notification_records
            .into_boxed()
            .select(NotificationRecordModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "channel_type" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(channel_type.eq(value));
                    }
                }
                "status" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(status.eq(value));
                    }
                }
                "recipient" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(recipient.eq(value));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<NotificationRecordModel>(conn) {
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

impl NotificationRecordModel {
    pub fn new(
        info: &NotificationRecordInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::insert_into(notification_records)
            .values(info)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(
        info: &NotificationRecordInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(notification_records.filter(id.eq(info.id.unwrap())))
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
        match diesel::delete(notification_records.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {} not found", _id))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
