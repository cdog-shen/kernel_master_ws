use chrono::{self, Local};
use diesel::{prelude::*, result::Error::NotFound};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::models::schema::light_ecs_table::{self, dsl::*};

static NOT_FOUND_CODE: u8 = 1;
static UNKNOW_ERROR_CODE: u8 = 0;
// static TMI_ERROR_CODE: u8 = 2;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = light_ecs_table)]
pub struct LightEcsModel {
    pub id: u64,
    pub project: String,
    pub cloud_name: String,
    pub region: String,
    pub zone: String,
    pub instance_id: String,
    pub instance_name: String,
    pub wip: Option<String>,
    pub nip: String,
    pub vpc_id: Option<String>,
    pub subnet_id: Option<String>,
    pub instance_type: Option<String>,
    pub internet_charge_type: String,
    pub status: String,
    pub os_name: String,
    pub os_type: String,
    pub image_id: String,
    pub bandwidth: String,
    pub cloud_account: String,
    pub is_link_server: bool,
    pub create_at: Option<chrono::NaiveDateTime>,
    pub update_at: Option<chrono::NaiveDateTime>,
}

impl LightEcsModel {
    fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(LightEcsModel {
            id: map.get("id").and_then(Value::as_u64).ok_or("id missing")?,
            project: map
                .get("project")
                .and_then(Value::as_str)
                .ok_or("project missing")?
                .to_string(),
            cloud_name: map
                .get("cloud_name")
                .and_then(Value::as_str)
                .ok_or("cloud_name missing")?
                .to_string(),
            region: map
                .get("region")
                .and_then(Value::as_str)
                .ok_or("region missing")?
                .to_string(),
            zone: map
                .get("zone")
                .and_then(Value::as_str)
                .ok_or("zone missing")?
                .to_string(),
            instance_id: map
                .get("instance_id")
                .and_then(Value::as_str)
                .ok_or("instance_id missing")?
                .to_string(),
            instance_name: map
                .get("instanÏce_name")
                .and_then(Value::as_str)
                .ok_or("instance_name missing")?
                .to_string(),
            wip: map.get("wip").and_then(Value::as_str).map(String::from),
            nip: map
                .get("nip")
                .and_then(Value::as_str)
                .ok_or("nip missing")?
                .to_string(),
            vpc_id: map.get("vpc_id").and_then(Value::as_str).map(String::from),
            subnet_id: map
                .get("subnet_id")
                .and_then(Value::as_str)
                .map(String::from),
            instance_type: map
                .get("instance_type")
                .and_then(Value::as_str)
                .map(String::from),
            internet_charge_type: map
                .get("internet_charge_type")
                .and_then(Value::as_str)
                .ok_or("internet_charge_type missing")?
                .to_string(),
            status: map
                .get("status")
                .and_then(Value::as_str)
                .ok_or("status missing")?
                .to_string(),
            os_name: map
                .get("os_name")
                .and_then(Value::as_str)
                .ok_or("os_name missing")?
                .to_string(),
            os_type: map
                .get("os_type")
                .and_then(Value::as_str)
                .ok_or("os_type missing")?
                .to_string(),
            image_id: map
                .get("image_id")
                .and_then(Value::as_str)
                .ok_or("image_id missing")?
                .to_string(),
            bandwidth: map
                .get("bandwidth")
                .and_then(Value::as_str)
                .ok_or("bandwidth missing")?
                .to_string(),
            cloud_account: map
                .get("cloud_account")
                .and_then(Value::as_str)
                .ok_or("cloud_account missing")?
                .to_string(),
            is_link_server: map
                .get("is_link_server")
                .and_then(Value::as_bool)
                .ok_or("is_link_server missing")?,
            create_at: map
                .get("create_at")
                .and_then(Value::as_str)
                .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()),
            update_at: map
                .get("update_at")
                .and_then(Value::as_str)
                .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()),
        })
    }

    /// get user by id
    pub fn get_user_by_id(ecs_id: u64, conn: &mut MysqlConnection) -> Result<Value, (u8, String)> {
        match light_ecs_table
            .filter(id.eq(ecs_id))
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
        let mut query = light_ecs_table
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
                        let pattern = format!("%{}%", value);
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
        let ecs_info = match LightEcsModel::from_map(ecs_info) {
            Ok(info) => info,
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };
        match diesel::insert_into(light_ecs_table)
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
        let ecs_info = match LightEcsModel::from_map(ecs_info) {
            Ok(info) => info,
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };
        match diesel::update(light_ecs_table.filter(id.eq(ecs_info.id)))
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
        match diesel::delete(light_ecs_table.filter(id.eq(ecs_info["id"].as_u64().unwrap())))
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
