use chrono::{self, Local};
use diesel::{prelude::*, result::Error::NotFound};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::models::schema::lighthouse_instance::{self, dsl::*};

static NOT_FOUND_CODE: u8 = 1;
static UNKNOW_ERROR_CODE: u8 = 0;
// static TMI_ERROR_CODE: u8 = 2;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = lighthouse_instance)]
pub struct LightEcsModel {
    pub id: u64,
    pub cloud_name: String,
    pub region: String,
    pub zone: String,
    pub instance_id: String,
    pub instance_name: String,
    pub wip: Option<String>,
    pub vpc_id: Option<String>,
    pub status: String,
    pub os_name: String,
    pub os_type: String,
    pub full_info: Option<String>,
    pub attach_info: Option<String>,
    pub create_at: Option<chrono::NaiveDateTime>,
    pub update_at: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = lighthouse_instance)]
pub struct LightEcsInfo {
    pub id: Option<u64>,
    pub cloud_name: Option<String>,
    pub region: Option<String>,
    pub zone: Option<String>,
    pub instance_id: Option<String>,
    pub instance_name: Option<String>,
    pub wip: Option<String>,
    pub vpc_id: Option<String>,
    pub status: Option<String>,
    pub os_name: Option<String>,
    pub os_type: Option<String>,
    pub full_info: Option<String>,
    pub attach_info: Option<String>,
    pub create_at: Option<chrono::NaiveDateTime>,
    pub update_at: Option<chrono::NaiveDateTime>,
}

impl LightEcsInfo {
    fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(LightEcsInfo {
            id: match map.get("id") {
                Some(value) => value.as_u64(),
                None => None,
            },

            cloud_name: match map.get("cloud_name") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            region: match map.get("region") {
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
            wip: match map.get("wip") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },

            vpc_id: match map.get("vpc_id") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },

            status: match map.get("status") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            os_name: match map.get("os_name") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            os_type: match map.get("os_type") {
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
            create_at: None,
            update_at: Some(Local::now().naive_local()),
        })
    }
}

impl LightEcsModel {
    /// get user by id
    pub fn get_user_by_id(ecs_id: u64, conn: &mut MysqlConnection) -> Result<Value, (u8, String)> {
        match lighthouse_instance
            .filter(id.eq(ecs_id))
            .select(LightEcsModel::as_select())
            .first::<LightEcsModel>(conn)
        {
            Ok(item_info) => {
                let mut item_map = serde_json::to_value(&item_info)
                    .unwrap()
                    .as_object()
                    .unwrap()
                    .clone();
                item_map.insert(
                    "create_at".to_string(),
                    Value::String(Local::now().naive_local().to_string()),
                );
                item_map.insert(
                    "update_at".to_string(),
                    Value::String(Local::now().naive_local().to_string()),
                );
                Ok(Value::Object(item_map))
            }
            Err(NotFound) => Err((NOT_FOUND_CODE, "Item not found".to_string())),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    /// get user full data
    pub fn get_user_info_with_filter(
        filter: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = lighthouse_instance
            .into_boxed()
            .select(LightEcsModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "cloud_name" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(cloud_name.eq(value));
                    }
                }
                "region" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(cloud_name.eq(value));
                    }
                }
                "os_type" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(os_type.eq(value));
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
            Ok(vec_item_info) => Ok(vec_item_info
                .into_iter()
                .map(|item| {
                    let mut item_map = serde_json::to_value(&item)
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .clone();
                    item_map.insert(
                        "create_at".to_string(),
                        Value::String(Local::now().naive_local().to_string()),
                    );
                    item_map.insert(
                        "update_at".to_string(),
                        Value::String(Local::now().naive_local().to_string()),
                    );
                    Value::Object(item_map)
                })
                .collect()),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}

impl LightEcsModel {
    ///new ecs
    pub fn new_ecs(
        ecs_info: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<usize, (u8, String)> {
        let ecs_info = match LightEcsInfo::from_map(ecs_info) {
            Ok(info) => info,
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };
        match diesel::insert_into(lighthouse_instance)
            .values(&ecs_info)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    /// update ecs by id
    pub fn update_ecs(
        ecs_info: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<usize, (u8, String)> {
        let ecs_info = match LightEcsInfo::from_map(ecs_info) {
            Ok(info) => info,
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };
        match diesel::update(lighthouse_instance.filter(id.eq(ecs_info.id.unwrap())))
            .set(&ecs_info)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    /// delete ecs by id
    pub fn delete_ecs(
        ecs_info: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::delete(lighthouse_instance.filter(id.eq(ecs_info["id"].as_u64().unwrap())))
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
