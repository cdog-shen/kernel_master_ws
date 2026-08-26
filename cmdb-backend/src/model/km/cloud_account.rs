use chrono::{self, Local};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::cloud_account::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = cloud_account)]
pub struct CloudAccountModel {
    pub id: i32,
    pub provider: String,
    pub nick_name: String,
    pub ak: String,
    pub sk: String,
    pub is_enable: bool,
    pub update_time: chrono::NaiveDateTime,
    pub comment: String,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = cloud_account)]
pub struct CloudAccountInfo {
    pub id: Option<i32>,
    pub provider: Option<String>,
    pub nick_name: Option<String>,
    pub ak: Option<String>,
    pub sk: Option<String>,
    pub is_enable: Option<bool>,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub comment: Option<String>,
}

impl CloudAccountInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(CloudAccountInfo {
            id: match map.get("id") {
                Some(value) => value.as_i64().map(|v| v as i32),
                None => None,
            },
            provider: match map.get("provider") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            nick_name: match map.get("nick_name") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            ak: match map.get("ak") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            sk: match map.get("sk") {
                Some(value) => value.as_str().map(|s| s.to_string()),
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

impl CloudAccountModel {
    pub fn get_model_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = cloud_account
            .into_boxed()
            .select(CloudAccountModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "provider" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(provider.eq(value));
                    }
                }
                "nick_name" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{value}%");
                        query = query.filter(nick_name.like(pattern));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<CloudAccountModel>(conn) {
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

impl CloudAccountModel {
    pub fn new(info: &CloudAccountInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::insert_into(cloud_account)
            .values(info)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(info: &CloudAccountInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::update(cloud_account.filter(id.eq(info.id.unwrap())))
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
        match diesel::delete(cloud_account.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {} not found", _id))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
