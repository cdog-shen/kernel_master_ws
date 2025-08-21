use chrono::{self, Local};
use diesel::{prelude::*, result::Error::NotFound};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::job_log::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = job_log)]
pub struct JobLogModel {
    pub id: String,
    pub script: String,
    pub exec_type: String,
    pub commander: String,
    pub worker: String,
    pub status: i16,
    pub params: serde_json::Value,
    pub result: String,
    pub finish_time: String,
    pub update_time: chrono::NaiveDateTime,
    pub comment: String,
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = job_log)]
pub struct JobLogInfo {
    pub id: Option<String>,
    pub script: Option<String>,
    pub exec_type: Option<String>,
    pub commander: Option<String>,
    pub worker: Option<String>,
    pub status: Option<i16>,
    pub params: Option<serde_json::Value>,
    pub result: Option<String>,
    pub finish_time: Option<String>,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub comment: Option<String>,
}

impl JobLogInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
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

            status: map.get("status").and_then(|v| v.as_i64().map(|i| i as i16)),

            params: map.get("params").cloned(),

            result: map
                .get("result")
                .and_then(|v| v.as_str().map(|s| s.to_string())),

            finish_time: map
                .get("finish_time")
                .and_then(|v| v.as_str().map(|s| s.to_string())),

            comment: map
                .get("comment")
                .and_then(|v| v.as_str().map(|s| s.to_string())),

            update_time: Some(Local::now().naive_local()),
        })
    }
}

impl JobLogModel {
    pub fn get_log_by_id(log_id: &String, conn: &mut PgConnection) -> Result<Self, (u8, String)> {
        job_log
            .filter(id.eq(log_id))
            .first::<JobLogModel>(conn)
            .map_err(|e| match e {
                _ => (UNKNOW_ERROR_CODE, format!("Unknow Error: {e}")),
            })
    }

    pub fn get_logs_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, (u8, String)> {
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
            Ok(vec_item_info) => Ok(vec_item_info),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn new_log(info: &JobLogInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::insert_into(job_log).values(info).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update_log(info: &JobLogInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::update(job_log.filter(id.eq(info.id.as_ref().unwrap())))
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

    pub fn delete_log(_id: &String, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(job_log.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(NotFound) => Err((BAD_REQUEST_CODE, format!("id: {_id} not found"))),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
