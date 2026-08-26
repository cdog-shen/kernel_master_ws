use chrono::{self, Local};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::cloudserver_instance::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = cloudserver_instance)]
pub struct CloudserverInstanceModel {
    pub id: i64,
    pub provider: String,
    pub zone: String,
    pub instance_id: String,
    pub instance_name: String,
    pub platform: String,
    pub status: String,
    pub private_ip: String,
    pub tag: serde_json::Value,
    pub full_info: serde_json::Value,
    pub attach_info: serde_json::Value,
    pub update_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = cloudserver_instance)]
pub struct CloudserverInstanceInfo {
    pub id: Option<i64>,
    pub provider: Option<String>,
    pub zone: Option<String>,
    pub instance_id: Option<String>,
    pub instance_name: Option<String>,
    pub platform: Option<String>,
    pub status: Option<String>,
    pub private_ip: Option<String>,
    pub tag: Option<serde_json::Value>,
    pub full_info: Option<serde_json::Value>,
    pub attach_info: Option<serde_json::Value>,
    pub update_at: Option<chrono::NaiveDateTime>,
}

impl CloudserverInstanceInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(CloudserverInstanceInfo {
            id: match map.get("id") {
                Some(value) => value.as_i64(),
                None => None,
            },
            provider: match map.get("provider") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            zone: match map.get("zone") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            instance_id: match map.get("instance_id") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            instance_name: match map.get("instance_name") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            platform: match map.get("platform") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            status: match map.get("status") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            private_ip: match map.get("private_ip") {
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

impl CloudserverInstanceModel {
    /// get model full data
    pub fn get_model_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = cloudserver_instance
            .into_boxed()
            .select(CloudserverInstanceModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "provider" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(provider.eq(value));
                    }
                }
                "zone" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{value}%");
                        query = query.filter(zone.like(pattern));
                    }
                }
                "platform" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(platform.eq(value));
                    }
                }
                "instance_id" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(instance_id.eq(value));
                    }
                }
                "status" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(status.eq(value));
                    }
                }
                "instance_name" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{value}%");
                        query = query.filter(instance_name.like(pattern));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<CloudserverInstanceModel>(conn) {
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

impl CloudserverInstanceModel {
    pub fn new(
        info: &CloudserverInstanceInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::insert_into(cloudserver_instance)
            .values(info)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(
        info: &CloudserverInstanceInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(cloudserver_instance.filter(id.eq(info.id.unwrap())))
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
        match diesel::delete(cloudserver_instance.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {} not found", _id))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
