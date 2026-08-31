use chrono::{self, Local};
use diesel::{PgConnection, prelude::*, result::Error::NotFound};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::subsystem_table::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

/// The structure of the Subsystem meta data in the database.
/// - uuid
///
///     Subsystem uuid, used as route (String)
///
/// - url
///
///     Subsystem target URL (URL String)
///
/// - is_enable
///
///    is this subsystem in use (tinyint 1/0)
///
/// - update_time
///
///     update time (datetime %Y-%m-%d %H:%M:%S)
///
/// - relate_service
///
///     subsystem combine service id (u32)
#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = subsystem_table)]
pub struct SubsysModel {
    pub id: i32,
    #[diesel(column_name = subsys_name)]
    pub subsys_name: String,
    #[diesel(column_name = url)]
    pub url: String,
    #[diesel(column_name = is_enable)]
    pub is_enable: bool,
    #[diesel(column_name = relate_service_id)]
    pub relate_service_id: i32,
    #[diesel(column_name = update_time)]
    pub update_time: chrono::NaiveDateTime,
    #[diesel(column_name = token)]
    pub token: uuid::Uuid,
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = subsystem_table)]
pub struct SubsysInfo {
    pub id: Option<i32>,
    pub subsys_name: Option<String>,
    pub url: Option<String>,
    pub is_enable: Option<bool>,
    pub relate_service_id: Option<i32>,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub token: Option<uuid::Uuid>,
}

impl SubsysInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        let subsys_info = SubsysInfo {
            id: map.get("id").and_then(|v| v.as_i64().map(|i| i as i32)),

            subsys_name: map
                .get("subsys_name")
                .and_then(|v| v.as_str().map(|v| v.to_string())),

            url: map
                .get("url")
                .and_then(|v| v.as_str().map(|v| v.to_string())),

            is_enable: map.get("is_enable").and_then(|v| v.as_bool()),

            relate_service_id: map
                .get("relate_service_id")
                .and_then(|v| v.as_i64().map(|i| i as i32)),

            token: map
                .get("token")
                .and_then(|v| v.as_str())
                .and_then(|s| Some(s.parse::<uuid::Uuid>().expect("token MUST BE a valid UUID"))),

            update_time: Some(Local::now().naive_local()),
        };

        Ok(subsys_info)
    }
}

// query implement
impl SubsysModel {
    /// get all enable services
    pub fn get_all_enable(conn: &mut PgConnection) -> Result<Vec<Self>, (u8, String)> {
        match subsystem_table
            .filter(is_enable.eq(true))
            .select(SubsysModel::as_select())
            .get_results::<SubsysModel>(conn)
        {
            Ok(subsystem_table_data) => Ok(subsystem_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    pub fn get_meta_by_id(subsys_id: i32, conn: &mut PgConnection) -> Result<Self, (u8, String)> {
        match subsystem_table
            .find(subsys_id)
            .select(SubsysModel::as_select())
            .get_result::<SubsysModel>(conn)
        {
            Ok(subsys) => Ok(subsys),
            Err(NotFound) => Err((
                BAD_REQUEST_CODE,
                format!("can NOT find subsystem id: {}.", &subsys_id),
            )),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    /// get enable subsystem by token (subsys uuid)
    pub fn get_enable_by_token(
        subsys_uuid: &uuid::Uuid,
        conn: &mut PgConnection,
    ) -> Result<Self, (u8, String)> {
        match subsystem_table
            .filter(is_enable.eq(true))
            .filter(token.eq(subsys_uuid))
            .select(SubsysModel::as_select())
            .get_result::<SubsysModel>(conn)
        {
            Ok(subsys_info) => Ok(subsys_info),
            Err(NotFound) => Err((
                BAD_REQUEST_CODE,
                format!("can NOT find subsystem uuid: {}.", &subsys_uuid),
            )),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    pub fn get_enable_by_name(
        name: &String,
        conn: &mut PgConnection,
    ) -> Result<Self, (u8, String)> {
        match subsystem_table
            .filter(is_enable.eq(true))
            .filter(subsys_name.eq(name))
            .select(SubsysModel::as_select())
            .get_result::<SubsysModel>(conn)
        {
            Ok(subsys_info) => Ok(subsys_info),
            Err(NotFound) => Err((
                BAD_REQUEST_CODE,
                format!("can NOT find subsystem: {}.", &name),
            )),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    /// get all services
    pub fn get_all_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, (u8, String)> {
        let mut query = subsystem_table
            .into_boxed()
            .select(SubsysModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "subsys_name" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(subsys_name.eq(value));
                    }
                }
                "is_enable" => {
                    if let Some(value) = q_v.as_bool() {
                        query = query.filter(is_enable.eq(value));
                    }
                }
                "url" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{value}%");
                        query = query.filter(url.like(pattern));
                    }
                }
                "relate_service_id" => {
                    if let Some(value) = q_v.as_i64() {
                        query = query.filter(relate_service_id.eq(value as i32));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<SubsysModel>(conn) {
            Ok(subsystem_table_data) => Ok(subsystem_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }
}

// update implement
impl SubsysModel {
    pub fn new_meta(
        subsys_info: &SubsysInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::insert_into(subsystem_table)
            .values(subsys_info)
            .execute(conn)
        {
            Ok(num_of_change) => Ok(num_of_change),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    pub fn update_meta_by_id(
        subsys_info: &SubsysInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(subsystem_table.filter(id.eq(subsys_info.id.unwrap())))
            .set(subsys_info)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((
                    BAD_REQUEST_CODE,
                    format!("id: {} not found", subsys_info.id.unwrap()),
                )),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete_meta_by_id(_id: i32, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(subsystem_table.find(_id)).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(NotFound) => Err((BAD_REQUEST_CODE, format!("id: {_id} not found"))),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
