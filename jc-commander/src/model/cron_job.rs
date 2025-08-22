use chrono::Local;
use diesel::{prelude::*, result::Error::NotFound};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::cron_job::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;
// static TMI_ERROR_CODE: u8 = 2;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
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

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
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
            id: map
                .get("id")
                .and_then(|v| v.as_str().map(|s| s.to_string())),

            script: map
                .get("script")
                .and_then(|v| v.as_str().map(|s| s.to_string())),

            frequency: map.get("frequency").and_then(|v| v.as_i64()),

            times: map.get("frequency").and_then(|v| v.as_i64()),

            params: map.get("params").cloned(),

            comment: map
                .get("comment")
                .and_then(|v| v.as_str().map(|s| s.to_string())),

            is_enable: map.get("is_enable").and_then(|v| v.as_bool()),

            launch_at: map.get("launch_at").and_then(|v| {
                v.as_str().and_then(|s| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok()
                })
            }),

            update_time: Some(Local::now().naive_local()),
        })
    }
}

impl CronJobModel {
    pub fn get_crons_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
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
                "is_enable" => {
                    if let Some(value) = q_v.as_bool() {
                        query = query.filter(is_enable.eq(value));
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
    pub fn new_cron(info: &CronJobInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::insert_into(cron_job).values(info).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update_cron(info: &CronJobInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
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

    pub fn next_round(_id: &String, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::update(cron_job.filter(id.eq(_id)))
            .set(times.eq(times + 1))
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("id: {_id} not found"))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete_cron(_id: &String, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(cron_job.filter(id.eq(_id))).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(NotFound) => Err((BAD_REQUEST_CODE, format!("id: {_id} not found"))),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
