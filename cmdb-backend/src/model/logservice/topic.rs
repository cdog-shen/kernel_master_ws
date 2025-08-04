use chrono::{self, Local};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::logservice_topic::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = logservice_topic)]
pub struct LogServiceTopicModel {
    pub id: i64,
    pub provider: String,
    pub set_id: String,
    pub topic_id: String,
    pub topic_name: String,
    pub status: String,
    pub hot_period: i32,
    pub period: i32,
    pub index: bool,
    pub describes: String,
    pub tag: serde_json::Value,
    pub full_info: serde_json::Value,
    pub attach_info: serde_json::Value,
    pub update_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = logservice_topic)]
pub struct LogServiceTopicInfo {
    pub id: Option<i64>,
    pub provider: Option<String>,
    pub set_id: Option<String>,
    pub topic_id: Option<String>,
    pub topic_name: Option<String>,
    pub status: Option<String>,
    pub hot_period: Option<i32>,
    pub period: Option<i32>,
    pub index: Option<bool>,
    pub describes: Option<String>,
    pub tag: Option<serde_json::Value>,
    pub full_info: Option<serde_json::Value>,
    pub attach_info: Option<serde_json::Value>,
    pub update_at: Option<chrono::NaiveDateTime>,
}

impl LogServiceTopicInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(LogServiceTopicInfo {
            id: match map.get("id") {
                Some(value) => value.as_i64(),
                None => None,
            },
            provider: match map.get("provider") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            set_id: match map.get("set_id") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            topic_id: match map.get("topic_id") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            topic_name: match map.get("topic_id") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            status: match map.get("status") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            hot_period: match map.get("hot_period") {
                Some(value) => Some(value.as_i64().map_or(0, |i| i as i32)),
                None => None,
            },
            period: match map.get("period") {
                Some(value) => Some(value.as_i64().map_or(0, |i| i as i32)),
                None => None,
            },
            index: match map.get("index") {
                Some(value) => value.as_bool(),
                None => None,
            },
            describes: match map.get("describes") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            tag: map.get("tag").cloned(),
            full_info: map.get("full_info").cloned(),
            attach_info: map.get("attach_info").cloned(),
            update_at: Some(Local::now().naive_local()),
        })
    }
}

impl LogServiceTopicModel {
    /// get model full data
    pub fn get_model_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = logservice_topic
            .into_boxed()
            .select(LogServiceTopicModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "provider" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(provider.eq(value));
                    }
                }
                "set_id" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{value}%");
                        query = query.filter(set_id.like(pattern));
                    }
                }
                "topic_id" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{value}%");
                        query = query.filter(topic_id.like(pattern));
                    }
                }
                "topic_name" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{value}%");
                        query = query.filter(topic_name.like(pattern));
                    }
                }
                "status" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(status.eq(value));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<LogServiceTopicModel>(conn) {
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

impl LogServiceTopicModel {
    pub fn new(info: &LogServiceTopicInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::insert_into(logservice_topic)
            .values(info)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(
        info: &LogServiceTopicInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(logservice_topic.filter(id.eq(info.id.unwrap())))
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

    pub fn delete(_id: i64, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(logservice_topic.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {} not found", _id))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
