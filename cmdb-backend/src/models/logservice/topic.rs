use chrono::{self, Local};
use diesel::{prelude::*, result::Error::NotFound};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::models::schema::logservice_topic::{self, dsl::*};

static NOT_FOUND_CODE: u8 = 1;
static UNKNOW_ERROR_CODE: u8 = 0;
// static TMI_ERROR_CODE: u8 = 2;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = logservice_topic)]
pub struct LogServiceTopicModel {
    pub id: u64,
    pub provider: String,
    pub set_id: String,
    pub topic_id: String,
    pub topic_name: String,
    pub status: String,
    pub hot_period: u32,
    pub period: u32,
    pub index: i8,
    pub full_info: Option<String>,
    pub attach_info: Option<String>,
    pub update_at: Option<chrono::NaiveDateTime>,
    pub describes: String,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = logservice_topic)]
pub struct LogServiceTopicInfo {
    pub id: Option<u64>,
    pub provider: Option<String>,
    pub set_id: Option<String>,
    pub topic_id: Option<String>,
    pub topic_name: Option<String>,
    pub status: Option<String>,
    pub hot_period: Option<u32>,
    pub period: Option<u32>,
    pub index: Option<i8>,
    pub full_info: Option<String>,
    pub attach_info: Option<String>,
    pub update_at: Option<chrono::NaiveDateTime>,
    pub describes: Option<String>,
}

impl LogServiceTopicInfo {
    fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(LogServiceTopicInfo {
            id: match map.get("id") {
                Some(value) => value.as_u64(),
                None => None,
            },
            provider: match map.get("provider") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            set_id: match map.get("provider") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            topic_id: match map.get("provider") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            topic_name: match map.get("provider") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            status: match map.get("provider") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            hot_period: match map.get("hot_period") {
                Some(value) => value.as_u64().and_then(|v| u32::try_from(v).ok()),
                None => None,
            },
            period: match map.get("hot_period") {
                Some(value) => value.as_u64().and_then(|v| u32::try_from(v).ok()),
                None => None,
            },
            index: match map.get("hot_period") {
                Some(value) => value.as_i64().and_then(|v| i8::try_from(v).ok()),
                None => None,
            },
            full_info: match map.get("full_info") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            attach_info: match map.get("attach_info") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            update_at: Some(Local::now().naive_local()),
            describes: match map.get("describes") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
        })
    }
}

impl LogServiceTopicModel {
    /// get model full data
    pub fn get_model_info_with_filter(
        filter: Map<String, Value>,
        conn: &mut MysqlConnection,
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
                        let pattern = format!("%{}%", value);
                        query = query.filter(set_id.like(pattern));
                    }
                }
                "topic_id" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{}%", value);
                        query = query.filter(set_id.like(pattern));
                    }
                }
                "topic_name" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{}%", value);
                        query = query.filter(set_id.like(pattern));
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
    pub fn new(
        info: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<usize, (u8, String)> {
        let info = match LogServiceTopicInfo::from_map(info) {
            Ok(info) => info,
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };
        match diesel::insert_into(logservice_topic)
            .values(&info)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(
        info: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<usize, (u8, String)> {
        let info = match LogServiceTopicInfo::from_map(info) {
            Ok(info) => info,
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };
        match diesel::update(logservice_topic.filter(id.eq(info.id.unwrap())))
            .set(&info)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete(
        info: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::delete(logservice_topic.filter(id.eq(info["id"].as_u64().unwrap())))
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
