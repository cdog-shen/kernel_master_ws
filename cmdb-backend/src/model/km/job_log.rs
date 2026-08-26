use chrono::{self, Local};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::job_log::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
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
    pub finish_time: chrono::NaiveDateTime,
    pub update_time: chrono::NaiveDateTime,
    pub comment: String,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
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
    pub finish_time: Option<chrono::NaiveDateTime>,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub comment: Option<String>,
}

impl JobLogInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(JobLogInfo {
            id: match map.get("id") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            script: match map.get("script") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            exec_type: match map.get("exec_type") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            commander: match map.get("commander") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            worker: match map.get("worker") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            status: match map.get("status") {
                Some(value) => value.as_i64().map(|v| v as i16),
                None => None,
            },
            params: map.get("params").cloned(),
            result: match map.get("result") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            finish_time: match map.get("finish_time") {
                Some(value) => value.as_str().and_then(|s| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()
                }),
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

impl JobLogModel {
    pub fn get_model_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = job_log.into_boxed().select(JobLogModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
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

impl JobLogModel {
    pub fn new(info: &JobLogInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::insert_into(job_log).values(info).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(info: &JobLogInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
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

    pub fn delete(_id: &str, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(job_log.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {} not found", _id))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
