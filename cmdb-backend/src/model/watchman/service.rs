use chrono::{self, Local};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::service_table::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = service_table)]
pub struct ServiceModel {
    pub id: i32,
    pub service_name: String,
    pub nick_name: String,
    pub service_point: String,
    pub is_enable: bool,
    pub update_time: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = service_table)]
pub struct ServiceInfo {
    pub id: Option<i32>,
    pub service_name: Option<String>,
    pub nick_name: Option<String>,
    pub service_point: Option<String>,
    pub is_enable: Option<bool>,
    pub update_time: Option<chrono::NaiveDateTime>,
}

impl ServiceInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(ServiceInfo {
            id: match map.get("id") {
                Some(value) => value.as_i64().map(|v| v as i32),
                None => None,
            },
            service_name: match map.get("service_name") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            nick_name: match map.get("nick_name") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            service_point: match map.get("service_point") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            is_enable: match map.get("is_enable") {
                Some(value) => value.as_bool(),
                None => None,
            },
            update_time: Some(Local::now().naive_local()),
        })
    }
}

impl ServiceModel {
    pub fn get_model_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = service_table
            .into_boxed()
            .select(ServiceModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "service_name" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(service_name.eq(value));
                    }
                }
                "service_point" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(service_point.eq(value));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<ServiceModel>(conn) {
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

impl ServiceModel {
    pub fn new(
        info: &ServiceInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::insert_into(service_table)
            .values(info)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(
        info: &ServiceInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(service_table.filter(id.eq(info.id.unwrap())))
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
        match diesel::delete(service_table.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {} not found", _id))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
