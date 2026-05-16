use chrono::{self, Local};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::webhook_table::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = webhook_table)]
pub struct WebhookModel {
    pub id: i32,
    pub token: String,
    pub hook_name: String,
    pub method_type: String,
    pub target_url: String,
    pub query_json: serde_json::Value,
    pub header_json: serde_json::Value,
    pub body_json: serde_json::Value,
    pub ttl: i64,
    pub is_enable: bool,
    pub update_time: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = webhook_table)]
pub struct WebhookInfo {
    pub id: Option<i32>,
    pub token: Option<String>,
    pub hook_name: Option<String>,
    pub method_type: Option<String>,
    pub target_url: Option<String>,
    pub query_json: Option<serde_json::Value>,
    pub header_json: Option<serde_json::Value>,
    pub body_json: Option<serde_json::Value>,
    pub ttl: Option<i64>,
    pub is_enable: Option<bool>,
    pub update_time: Option<chrono::NaiveDateTime>,
}

impl WebhookInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(WebhookInfo {
            id: match map.get("id") {
                Some(value) => value.as_i64().map(|v| v as i32),
                None => None,
            },
            token: match map.get("token") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            hook_name: match map.get("hook_name") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            method_type: match map.get("method_type") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            target_url: match map.get("target_url") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            query_json: map.get("query_json").cloned(),
            header_json: map.get("header_json").cloned(),
            body_json: map.get("body_json").cloned(),
            ttl: match map.get("ttl") {
                Some(value) => value.as_i64(),
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

impl WebhookModel {
    pub fn get_model_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = webhook_table
            .into_boxed()
            .select(WebhookModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "hook_name" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(hook_name.eq(value));
                    }
                }
                "method_type" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(method_type.eq(value));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<WebhookModel>(conn) {
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

impl WebhookModel {
    pub fn new(
        info: &WebhookInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::insert_into(webhook_table)
            .values(info)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(
        info: &WebhookInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(webhook_table.filter(id.eq(info.id.unwrap())))
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
        match diesel::delete(webhook_table.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {} not found", _id))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
