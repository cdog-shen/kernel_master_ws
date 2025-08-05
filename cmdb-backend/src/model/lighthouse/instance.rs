use chrono::{self, Local};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::lighthouse_instance::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = lighthouse_instance)]
pub struct LightEcsModel {
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

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = lighthouse_instance)]
pub struct LightEcsInfo {
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

impl LightEcsInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(LightEcsInfo {
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
            platform: match map.get("plantform") {
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

impl LightEcsModel {
    pub fn get_model_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<LightEcsModel>, (u8, String)> {
        let mut query = lighthouse_instance
            .into_boxed()
            .select(LightEcsModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "provider" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(provider.eq(value));
                    }
                }
                "zone" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(zone.eq(value));
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
                "instance_name" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{value}%");
                        query = query.filter(instance_name.like(pattern));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<LightEcsModel>(conn) {
            Ok(vec_item_info) => Ok(vec_item_info),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}

impl LightEcsModel {
    ///new ecs
    pub fn new_ecs(
        ecs_info: &LightEcsInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::insert_into(lighthouse_instance)
            .values(ecs_info)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    /// update ecs by id
    pub fn update_ecs(
        ecs_info: &LightEcsInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(lighthouse_instance.filter(id.eq(ecs_info.id.unwrap())))
            .set(ecs_info)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    /// delete ecs by id
    pub fn delete_ecs(_id: i64, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(lighthouse_instance.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {} not found", _id))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
