use chrono::{self, Local};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::access_table::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = access_table)]
pub struct AccessModel {
    pub id: i32,
    pub service_id: i32,
    pub group_id: i32,
    pub group_access: i16,
    pub is_enable: bool,
    pub update_time: chrono::NaiveDateTime,
    pub comment: String,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = access_table)]
pub struct AccessInfo {
    pub id: Option<i32>,
    pub service_id: Option<i32>,
    pub group_id: Option<i32>,
    pub group_access: Option<i16>,
    pub is_enable: Option<bool>,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub comment: Option<String>,
}

impl AccessInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(AccessInfo {
            id: match map.get("id") {
                Some(value) => value.as_i64().map(|v| v as i32),
                None => None,
            },
            service_id: match map.get("service_id") {
                Some(value) => value.as_i64().map(|v| v as i32),
                None => None,
            },
            group_id: match map.get("group_id") {
                Some(value) => value.as_i64().map(|v| v as i32),
                None => None,
            },
            group_access: match map.get("group_access") {
                Some(value) => value.as_i64().map(|v| v as i16),
                None => None,
            },
            is_enable: match map.get("is_enable") {
                Some(value) => value.as_bool(),
                None => None,
            },
            update_time: Some(Local::now().naive_local()),
            comment: match map.get("comment") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
        })
    }
}

impl AccessModel {
    pub fn get_model_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = access_table.into_boxed().select(AccessModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "service_id" => {
                    if let Some(value) = q_v.as_i64() {
                        query = query.filter(service_id.eq(value as i32));
                    }
                }
                "group_id" => {
                    if let Some(value) = q_v.as_i64() {
                        query = query.filter(group_id.eq(value as i32));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<AccessModel>(conn) {
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

impl AccessModel {
    pub fn new(info: &AccessInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::insert_into(access_table).values(info).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(info: &AccessInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::update(access_table.filter(id.eq(info.id.unwrap())))
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
        match diesel::delete(access_table.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {} not found", _id))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
