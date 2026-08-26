use chrono::{self, Local};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::cloudstorage_bucket::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = cloudstorage_bucket)]
pub struct BucketModel {
    pub id: i64,
    pub provider: String,
    pub bucket_name: String,
    pub bucket_type: String,
    pub location: String,
    pub creation_date: String,
    pub describes: String,
    pub tag: serde_json::Value,
    pub full_info: serde_json::Value,
    pub attach_info: serde_json::Value,
    pub update_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = cloudstorage_bucket)]
pub struct BucketInfo {
    pub id: Option<i64>,
    pub provider: Option<String>,
    pub bucket_name: Option<String>,
    pub bucket_type: Option<String>,
    pub location: Option<String>,
    pub creation_date: Option<String>,
    pub describes: Option<String>,
    pub tag: Option<serde_json::Value>,
    pub full_info: Option<serde_json::Value>,
    pub attach_info: Option<serde_json::Value>,
    pub update_at: Option<chrono::NaiveDateTime>,
}

impl BucketInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(BucketInfo {
            id: match map.get("id") {
                Some(value) => value.as_i64(),
                None => None,
            },
            provider: match map.get("provider") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            bucket_name: match map.get("bucket_name") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            bucket_type: match map.get("bucket_type") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            location: match map.get("location") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            creation_date: match map.get("creation_date") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            describes: match map.get("describes") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            tag: map.get("tag").cloned(),
            full_info: map.get("full_info").cloned(),
            attach_info: map.get("attach_info").cloned(),
            update_at: Some(Local::now().naive_local()),
        })
    }
}

impl BucketModel {
    pub fn get_model_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = cloudstorage_bucket
            .into_boxed()
            .select(BucketModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "provider" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(provider.eq(value));
                    }
                }
                "bucket_name" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{value}%");
                        query = query.filter(bucket_name.like(pattern));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<BucketModel>(conn) {
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

impl BucketModel {
    pub fn new(info: &BucketInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::insert_into(cloudstorage_bucket)
            .values(info)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(info: &BucketInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::update(cloudstorage_bucket.filter(id.eq(info.id.unwrap())))
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

    pub fn delete(_id: i64, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(cloudstorage_bucket.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {} not found", _id))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
