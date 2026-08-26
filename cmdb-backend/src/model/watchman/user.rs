use chrono::{self, Local};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::user_table::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = user_table)]
pub struct UserModel {
    pub id: i32,
    pub username: String,
    pub passwd: String,
    pub is_enable: bool,
    pub full_name: String,
    pub contact: serde_json::Value,
    pub last_login: chrono::NaiveDateTime,
    pub update_time: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = user_table)]
pub struct UserInfo {
    pub id: Option<i32>,
    pub username: Option<String>,
    pub passwd: Option<String>,
    pub is_enable: Option<bool>,
    pub full_name: Option<String>,
    pub contact: Option<serde_json::Value>,
    pub last_login: Option<chrono::NaiveDateTime>,
    pub update_time: Option<chrono::NaiveDateTime>,
}

impl UserInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(UserInfo {
            id: match map.get("id") {
                Some(value) => value.as_i64().map(|v| v as i32),
                None => None,
            },
            username: match map.get("username") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            passwd: match map.get("passwd") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            is_enable: match map.get("is_enable") {
                Some(value) => value.as_bool(),
                None => None,
            },
            full_name: match map.get("full_name") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            contact: map.get("contact").cloned(),
            last_login: match map.get("last_login") {
                Some(value) => value.as_str().and_then(|s| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()
                }),
                None => None,
            },
            update_time: Some(Local::now().naive_local()),
        })
    }
}

impl UserModel {
    pub fn get_model_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = user_table.into_boxed().select(UserModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "username" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(username.eq(value));
                    }
                }
                "full_name" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{value}%");
                        query = query.filter(full_name.like(pattern));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<UserModel>(conn) {
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

impl UserModel {
    pub fn new(info: &UserInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::insert_into(user_table).values(info).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(info: &UserInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::update(user_table.filter(id.eq(info.id.unwrap())))
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
        match diesel::delete(user_table.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {} not found", _id))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
