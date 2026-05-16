use chrono::{self};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::notification_group_members::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = notification_group_members)]
pub struct NotificationGroupMemberModel {
    pub id: i32,
    pub group_id: i32,
    pub channel_type: String,
    pub recipient: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = notification_group_members)]
pub struct NotificationGroupMemberInfo {
    pub id: Option<i32>,
    pub group_id: Option<i32>,
    pub channel_type: Option<String>,
    pub recipient: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

impl NotificationGroupMemberInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(NotificationGroupMemberInfo {
            id: match map.get("id") {
                Some(value) => value.as_i64().map(|v| v as i32),
                None => None,
            },
            group_id: match map.get("group_id") {
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
            created_at: match map.get("created_at") {
                Some(_) => None,
                None => None,
            },
        })
    }
}

impl NotificationGroupMemberModel {
    /// get model full data
    pub fn get_model_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = notification_group_members
            .into_boxed()
            .select(NotificationGroupMemberModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "group_id" => {
                    if let Some(value) = q_v.as_i64() {
                        query = query.filter(group_id.eq(value as i32));
                    }
                }
                "channel_type" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(channel_type.eq(value));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<NotificationGroupMemberModel>(conn) {
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

impl NotificationGroupMemberModel {
    pub fn new(
        info: &NotificationGroupMemberInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::insert_into(notification_group_members)
            .values(info)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(
        info: &NotificationGroupMemberInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(notification_group_members.filter(id.eq(info.id.unwrap())))
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
        match diesel::delete(notification_group_members.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {} not found", _id))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
