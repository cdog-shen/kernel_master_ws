use chrono::{self, Local};
use diesel::{prelude::*, result::Error::NotFound};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::models::schema::cloudserver_instance::{self, dsl::*};

static NOT_FOUND_CODE: u8 = 1;
static UNKNOW_ERROR_CODE: u8 = 0;
// static TMI_ERROR_CODE: u8 = 2;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = cloudserver_instance)]
pub struct CloudserverInstanceModel {
    pub id: u64,
    pub provider: String,
    pub zone: String,
    pub instance_id: String,
    pub instance_name: String,
    pub plantform: String,
    pub status: String,
    pub tag: String,
    pub private_ip: String,
    pub full_info: Option<String>,
    pub attach_info: Option<String>,
    pub update_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = cloudserver_instance)]
pub struct CloudserverInstanceInfo {
    pub id: Option<u64>,
    pub provider: Option<String>,
    pub zone: Option<String>,
    pub instance_id: Option<String>,
    pub instance_name: Option<String>,
    pub plantform: Option<String>,
    pub status: Option<String>,
    pub tag: Option<String>,
    pub private_ip: Option<String>,
    pub full_info: Option<String>,
    pub attach_info: Option<String>,
    pub update_at: Option<chrono::NaiveDateTime>,
}

impl CloudserverInstanceInfo {
    fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(CloudserverInstanceInfo {
            id: match map.get("id") {
                Some(value) => value.as_u64(),
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
            plantform: match map.get("plantform") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            status: match map.get("status") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            tag: match map.get("tag") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            private_ip: match map.get("private_ip") {
                Some(value) => value.as_str().map(|s| s.to_string()),
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
        })
    }
}

impl CloudserverInstanceModel {
    /// get model full data
    pub fn get_model_info_with_filter(
        filter: Map<String, Value>,
        conn: &mut MysqlConnection,
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
                "plantform" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(plantform.eq(value));
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
        info: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<usize, (u8, String)> {
        let info = match CloudserverInstanceInfo::from_map(info) {
            Ok(info) => info,
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };
        match diesel::insert_into(cloudserver_instance)
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
        let info = match CloudserverInstanceInfo::from_map(info) {
            Ok(info) => info,
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };
        match diesel::update(cloudserver_instance.filter(id.eq(info.id.unwrap())))
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
        match diesel::delete(cloudserver_instance.filter(id.eq(info["id"].as_u64().unwrap())))
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
