use chrono::{self, Local};
use diesel::{prelude::*, result::Error::NotFound};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::models::schema::job_log::{self, dsl::*};

static NOT_FOUND_CODE: u8 = 1;
static UNKNOW_ERROR_CODE: u8 = 0;
// static TMI_ERROR_CODE: u8 = 2;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = job_log)]
pub struct JobLogModel {
    pub id: String,
    pub script: String,
    pub exec_type: String,
    pub commander: String,
    pub worker: Option<String>,
    pub status: u8,
    pub params: String,
    pub result: String,
    pub create_time: Option<chrono::NaiveDateTime>,
    pub finish_time: Option<chrono::NaiveDateTime>,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub comment: Option<String>,
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = job_log)]
pub struct JobLogInfo {
    pub id: Option<String>,
    pub script: Option<String>,
    pub exec_type: Option<String>,
    pub commander: Option<String>,
    pub worker: Option<String>,
    pub status: Option<u8>,
    pub params: Option<String>,
    pub result: Option<String>,
    pub create_time: Option<chrono::NaiveDateTime>,
    pub finish_time: Option<chrono::NaiveDateTime>,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub comment: Option<String>,
}

impl JobLogInfo {
    fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(JobLogInfo {
            id: map
                .get("id")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            script: map
                .get("script")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            exec_type: map
                .get("exec_type")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            commander: map
                .get("commander")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            worker: map
                .get("worker")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            status: map.get("status").and_then(|v| v.as_u64().map(|s| s as u8)),
            params: map
                .get("params")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            result: map
                .get("result")
                .and_then(|v| v.as_str().map(|s| s.to_string())), // Added field
            create_time: map.get("create_time").and_then(|v| {
                v.as_str().and_then(|s| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok()
                })
            }),
            finish_time: map.get("finish_time").and_then(|v| {
                v.as_str().and_then(|s| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok()
                })
            }),
            update_time: Some(Local::now().naive_local()),
            comment: map
                .get("comment")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        })
    }
}

impl JobLogModel {
    pub fn get_log_by_id(log_id: String, conn: &mut MysqlConnection) -> Result<Self, (u8, String)> {
        job_log
            .filter(id.eq(log_id))
            .first::<JobLogModel>(conn)
            .map_err(|e| match e {
                NotFound => (NOT_FOUND_CODE, "log not found".to_string()),
                _ => (UNKNOW_ERROR_CODE, format!("get log error: {e}")),
            })
    }

    pub fn get_logs_with_filter(
        filter: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = job_log.into_boxed().select(JobLogModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "id" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(id.eq(value));
                    }
                }
                "exec_type" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(exec_type.eq(value));
                    }
                }
                "worker" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(worker.eq(value));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<JobLogModel>(conn) {
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

    pub fn new_log(
        log_map: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<usize, (u8, String)> {
        let new_log = match JobLogInfo::from_map(log_map) {
            Ok(log) => log,
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };
        match diesel::insert_into(job_log).values(&new_log).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update_log(
        log_map: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<usize, (u8, String)> {
        let update_log = match JobLogInfo::from_map(log_map) {
            Ok(log) => {
                if let Some(ref _uid) = log.id {
                    log
                } else {
                    return Err((UNKNOW_ERROR_CODE, "id is required".to_string()));
                }
            }
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };
        match diesel::update(job_log.filter(id.eq(update_log.id.as_ref().unwrap())))
            .set(&update_log)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete_log(log_id: String, conn: &mut MysqlConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(job_log.filter(id.eq(log_id))).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
