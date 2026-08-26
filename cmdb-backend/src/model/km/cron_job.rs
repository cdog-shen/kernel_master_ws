use chrono::{self, Local};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::cron_job::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = cron_job)]
pub struct CronJobModel {
    pub id: String,
    pub script: String,
    pub frequency: i64,
    pub times: i64,
    pub params: serde_json::Value,
    pub comment: String,
    pub is_enable: bool,
    pub launch_at: chrono::NaiveDateTime,
    pub update_time: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = cron_job)]
pub struct CronJobInfo {
    pub id: Option<String>,
    pub script: Option<String>,
    pub frequency: Option<i64>,
    pub times: Option<i64>,
    pub params: Option<serde_json::Value>,
    pub comment: Option<String>,
    pub is_enable: Option<bool>,
    pub launch_at: Option<chrono::NaiveDateTime>,
    pub update_time: Option<chrono::NaiveDateTime>,
}

impl CronJobInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(CronJobInfo {
            id: match map.get("id") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            script: match map.get("script") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            frequency: match map.get("frequency") {
                Some(value) => value.as_i64(),
                None => None,
            },
            times: match map.get("times") {
                Some(value) => value.as_i64(),
                None => None,
            },
            params: map.get("params").cloned(),
            comment: match map.get("comment") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            is_enable: match map.get("is_enable") {
                Some(value) => value.as_bool(),
                None => None,
            },
            launch_at: match map.get("launch_at") {
                Some(value) => value.as_str().and_then(|s| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()
                }),
                None => None,
            },
            update_time: Some(Local::now().naive_local()),
        })
    }
}

impl CronJobModel {
    pub fn get_model_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = cron_job.into_boxed().select(CronJobModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "id" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(id.eq(value));
                    }
                }
                "script" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{value}%");
                        query = query.filter(script.like(pattern));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<CronJobModel>(conn) {
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

impl CronJobModel {
    pub fn new(info: &CronJobInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::insert_into(cron_job).values(info).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(info: &CronJobInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::update(cron_job.filter(id.eq(info.id.as_ref().unwrap())))
            .set(info)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((
                    BAD_REQUEST_CODE,
                    format!("id: {} not found", info.id.as_ref().unwrap()),
                )),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete(_id: &str, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(cron_job.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {} not found", _id))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
