use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::models::schema::cron_job::{self, dsl::*};

// static NOT_FOUND_CODE: u8 = 1;
static UNKNOW_ERROR_CODE: u8 = 0;
// static TMI_ERROR_CODE: u8 = 2;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = cron_job)]
pub struct CronJobModel {
    pub id: String,
    pub script: String,
    pub frequency: i64,
    pub launch_at: chrono::NaiveDateTime,
    pub times: u32,
    pub status: u8,
    pub params: String,
    pub create_time: chrono::NaiveDateTime,
    pub update_time: chrono::NaiveDateTime,
    pub comment: String,
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = cron_job)]
pub struct CronJobInfo {
    pub id: Option<String>,
    pub script: Option<String>,
    pub frequency: Option<i64>,
    pub launch_at: Option<chrono::NaiveDateTime>,
    pub times: Option<u32>,
    pub status: Option<u8>,
    pub params: Option<String>,
    pub create_time: Option<chrono::NaiveDateTime>,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub comment: Option<String>,
}

impl CronJobInfo {
    fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(CronJobInfo {
            id: map
                .get("id")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            script: map
                .get("script")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            frequency: map.get("frequency").and_then(|v| v.as_i64()),
            launch_at: map
                .get("launch_at")
                .and_then(|v| v.as_str())
                .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()),
            times: Some(0),
            status: map.get("status").and_then(|v| v.as_u64()).map(|v| v as u8),
            params: map
                .get("params")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            create_time: map
                .get("create_time")
                .and_then(|v| v.as_str())
                .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()),
            update_time: map
                .get("update_time")
                .and_then(|v| v.as_str())
                .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()),
            comment: map
                .get("comment")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        })
    }
}

impl CronJobModel {
    pub fn get_crons_with_filter(
        filter: Map<String, Value>,
        conn: &mut MysqlConnection,
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
                        query = query.filter(script.eq(value));
                    }
                }
                "status" => {
                    if let Ok(value) = q_v.as_str().unwrap().parse::<u8>() {
                        query = query.filter(status.eq(value));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<CronJobModel>(conn) {
            Ok(vec_item_info) => Ok(vec_item_info
                .into_iter()
                .map(|item| {
                    let item_map = serde_json::to_value(&item)
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .clone();
                    Value::Object(item_map)
                })
                .collect()),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn get_crons_obj_with_filter(
        filter: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<Self>, (u8, String)> {
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
                        query = query.filter(script.eq(value));
                    }
                }
                "status" => {
                    if let Some(value) = q_v.as_number() {
                        query = query.filter(status.eq(value.as_u64().unwrap() as u8));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<CronJobModel>(conn) {
            Ok(vec_item_info) => Ok(vec_item_info.into_iter().collect()),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}

impl CronJobModel {
    pub fn new_cron(
        cron_map: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<usize, (u8, String)> {
        let new_cron = match CronJobInfo::from_map(cron_map) {
            Ok(cron) => cron,
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };
        match diesel::insert_into(cron_job)
            .values(&new_cron)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update_cron(
        cron_map: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<usize, (u8, String)> {
        let update_cron = match CronJobInfo::from_map(cron_map) {
            Ok(cron) => {
                if let Some(ref _uid) = cron.id {
                    cron
                } else {
                    return Err((UNKNOW_ERROR_CODE, "id is required".to_string()));
                }
            }
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };
        match diesel::update(cron_job.filter(id.eq(update_cron.id.as_ref().unwrap())))
            .set(&update_cron)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn next_round(cron_id: &String, conn: &mut MysqlConnection) -> Result<usize, (u8, String)> {
        match diesel::update(cron_job.filter(id.eq(cron_id)))
            .set(times.eq(times + 1))
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete_cron(cron_id: String, conn: &mut MysqlConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(cron_job.filter(id.eq(cron_id))).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
