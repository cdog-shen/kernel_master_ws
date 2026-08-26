use chrono::{self, Local};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::group_table::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = group_table)]
pub struct GroupModel {
    pub id: i32,
    pub group_name: String,
    pub is_enable: bool,
    pub user_ids: serde_json::Value,
    pub update_time: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = group_table)]
pub struct GroupInfo {
    pub id: Option<i32>,
    pub group_name: Option<String>,
    pub is_enable: Option<bool>,
    pub user_ids: Option<serde_json::Value>,
    pub update_time: Option<chrono::NaiveDateTime>,
}

impl GroupInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(GroupInfo {
            id: match map.get("id") {
                Some(value) => value.as_i64().map(|v| v as i32),
                None => None,
            },
            group_name: match map.get("group_name") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            is_enable: match map.get("is_enable") {
                Some(value) => value.as_bool(),
                None => None,
            },
            user_ids: map.get("user_ids").cloned(),
            update_time: Some(Local::now().naive_local()),
        })
    }
}

impl GroupModel {
    pub fn get_model_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = group_table.into_boxed().select(GroupModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "group_name" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{value}%");
                        query = query.filter(group_name.like(pattern));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<GroupModel>(conn) {
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

impl GroupModel {
    pub fn new(info: &GroupInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::insert_into(group_table).values(info).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(info: &GroupInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::update(group_table.filter(id.eq(info.id.unwrap())))
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
        match diesel::delete(group_table.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {} not found", _id))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
