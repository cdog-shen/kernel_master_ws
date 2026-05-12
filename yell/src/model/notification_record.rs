use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::notification_records::{self, dsl::*};

static UNKNOWN_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize)]
#[diesel(table_name = notification_records, check_for_backend(diesel::pg::Pg))]
pub struct NotificationRecord {
    pub id: i32,
    pub channel_type: String,
    pub recipient: String,
    pub subject: Option<String>,
    pub content: String,
    pub content_format: Option<String>,
    pub template_id: Option<i32>,
    pub status: Option<String>,
    pub error_msg: Option<String>,
    pub sent_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = notification_records)]
pub struct NewNotificationRecord {
    pub channel_type: String,
    pub recipient: String,
    pub subject: Option<String>,
    pub content: String,
    pub content_format: Option<String>,
    pub template_id: Option<i32>,
    pub status: Option<String>,
    pub error_msg: Option<String>,
    pub sent_at: Option<NaiveDateTime>,
}

#[derive(AsChangeset, Debug, Serialize, Deserialize)]
#[diesel(table_name = notification_records)]
pub struct UpdateNotificationRecord {
    pub status: Option<String>,
    pub error_msg: Option<String>,
    pub sent_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl NotificationRecord {
    /// 根据 ID 获取通知记录
    pub fn get_by_id(record_id: i32, conn: &mut PgConnection) -> Result<Option<Self>, (u8, String)> {
        match notification_records
            .filter(id.eq(record_id))
            .first::<NotificationRecord>(conn)
        {
            Ok(record) => Ok(Some(record)),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 根据过滤器获取通知记录列表
    pub fn get_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = notification_records
            .into_boxed()
            .select(NotificationRecord::as_select());

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

        match query.load::<NotificationRecord>(conn) {
            Ok(records) => Ok(records
                .into_iter()
                .map(|r| {
                    let value = serde_json::to_value(&r).unwrap();
                    value
                })
                .collect()),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 创建新的通知记录
    pub fn create(
        new_record: &NewNotificationRecord,
        conn: &mut PgConnection,
    ) -> Result<i32, (u8, String)> {
        match diesel::insert_into(notification_records)
            .values(new_record)
            .returning(id)
            .get_result::<i32>(conn)
        {
            Ok(new_id) => Ok(new_id),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 更新通知记录状态
    pub fn update_status(
        record_id: i32,
        update: &UpdateNotificationRecord,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(notification_records.filter(id.eq(record_id)))
            .set(update)
            .execute(conn)
        {
            Ok(num) => match num {
                0 => Err((BAD_REQUEST_CODE, format!("Record id: {} not found", record_id))),
                _ => Ok(num),
            },
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }
}
