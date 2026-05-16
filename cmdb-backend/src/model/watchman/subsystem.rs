use chrono::{self, Local};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::subsystem_table::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = subsystem_table)]
pub struct SubsystemModel {
    pub id: i32,
    pub subsys_name: String,
    pub url: String,
    pub is_enable: bool,
    pub relate_service_id: i32,
    pub token: uuid::Uuid,
    pub update_time: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = subsystem_table)]
pub struct SubsystemInfo {
    pub id: Option<i32>,
    pub subsys_name: Option<String>,
    pub url: Option<String>,
    pub is_enable: Option<bool>,
    pub relate_service_id: Option<i32>,
    pub token: Option<uuid::Uuid>,
    pub update_time: Option<chrono::NaiveDateTime>,
}

impl SubsystemInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(SubsystemInfo {
            id: match map.get("id") {
                Some(value) => value.as_i64().map(|v| v as i32),
                None => None,
            },
            subsys_name: match map.get("subsys_name") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            url: match map.get("url") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            is_enable: match map.get("is_enable") {
                Some(value) => value.as_bool(),
                None => None,
            },
            relate_service_id: match map.get("relate_service_id") {
                Some(value) => value.as_i64().map(|v| v as i32),
                None => None,
            },
            token: match map.get("token") {
                Some(value) => match value.as_str() {
                    Some(s) => match uuid::Uuid::parse_str(s) {
                        Ok(uuid) => Some(uuid),
                        Err(_) => None,
                    },
                    None => None,
                },
                None => None,
            },
            update_time: Some(Local::now().naive_local()),
        })
    }
}

impl SubsystemModel {
    pub fn get_model_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = subsystem_table
            .into_boxed()
            .select(SubsystemModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "subsys_name" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(subsys_name.eq(value));
                    }
                }
                "url" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(url.eq(value));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<SubsystemModel>(conn) {
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

impl SubsystemModel {
    pub fn new(
        info: &SubsystemInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::insert_into(subsystem_table)
            .values(info)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(
        info: &SubsystemInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(subsystem_table.filter(id.eq(info.id.unwrap())))
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
        match diesel::delete(subsystem_table.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {} not found", _id))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
