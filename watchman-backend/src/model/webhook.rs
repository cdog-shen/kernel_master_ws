use chrono::{self, Local};
use diesel::{
    Insertable, PgConnection, Queryable, Selectable, prelude::*, result::Error::NotFound,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::webhook_table::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = webhook_table)]
pub struct WebhookModel {
    pub id: i32,
    #[diesel(column_name = token)]
    pub token: String,
    #[diesel(column_name = hook_name)]
    pub hook_name: String,
    #[diesel(column_name = method_type)]
    pub method_type: String,
    #[diesel(column_name = target_url)]
    pub target_url: String,
    #[diesel(column_name = query_json)]
    pub query_json: serde_json::Value,
    #[diesel(column_name = header_json)]
    pub header_json: serde_json::Value,
    #[diesel(column_name = body_json)]
    pub body_json: serde_json::Value,
    #[diesel(column_name = ttl)]
    pub ttl: i64,
    #[diesel(column_name = is_enable)]
    pub is_enable: bool,
    #[diesel(column_name = update_time)]
    pub update_time: chrono::NaiveDateTime,
}

#[derive(
    Default, Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset,
)]
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
            id: map.get("id").and_then(|v| v.as_i64().map(|i| i as i32)),
            token: map
                .get("token")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            hook_name: map
                .get("hook_name")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            method_type: map
                .get("method_type")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            target_url: map
                .get("target_url")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            query_json: map.get("query_json").cloned(),
            header_json: map.get("header_json").cloned(),
            body_json: map.get("body_json").cloned(),
            ttl: map.get("ttl").and_then(|v| v.as_i64()),
            is_enable: map.get("is_enable").and_then(|v| v.as_bool()),
            update_time: Some(Local::now().naive_local()),
        })
    }
}

// query implement
impl WebhookModel {
    /// get all enable
    pub fn get_all_enable(conn: &mut PgConnection) -> Result<Vec<Self>, (u8, String)> {
        match webhook_table
            .filter(is_enable.eq(true))
            .filter(ttl.gt(0))
            .select(WebhookModel::as_select())
            .get_results::<WebhookModel>(conn)
        {
            Ok(webhook_table_data) => Ok(webhook_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    /// get all
    pub fn get_all_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, (u8, String)> {
        let mut query = webhook_table.into_boxed().select(WebhookModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "hook_name" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(hook_name.like(format!("%{value}%")));
                    }
                }
                "is_enable" => {
                    if let Some(value) = q_v.as_bool() {
                        query = query.filter(is_enable.eq(value));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<WebhookModel>(conn) {
            Ok(webhook_table_data) => Ok(webhook_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    /// get enable by token
    pub fn get_enable_by_token(
        _token: &str,
        conn: &mut PgConnection,
    ) -> Result<Self, (u8, String)> {
        match webhook_table
            .filter(token.eq(_token))
            .select(WebhookModel::as_select())
            .first(conn)
        {
            Ok(webhook_table_data) => Ok(webhook_table_data),
            Err(NotFound) => Err((BAD_REQUEST_CODE, format!("token: {_token} not found"))),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}

// update implement
impl WebhookModel {
    pub fn new(webhook_info: &WebhookInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::insert_into(webhook_table)
            .values(webhook_info)
            .execute(conn)
        {
            Ok(num_of_change) => Ok(num_of_change),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    pub fn update(
        webhook_info: &WebhookInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(webhook_table.filter(id.eq(webhook_info.id.unwrap())))
            .set(webhook_info)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((
                    BAD_REQUEST_CODE,
                    format!("id: {} not found", webhook_info.id.unwrap()),
                )),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete(_id: i32, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(webhook_table.find(_id)).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(NotFound) => Err((BAD_REQUEST_CODE, format!("id: {_id} not found"))),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
