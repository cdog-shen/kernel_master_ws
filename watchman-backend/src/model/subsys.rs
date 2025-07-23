use chrono::{self, Local};
use diesel::{prelude::*, result::Error::NotFound, MysqlConnection};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::subsystem_table::{self, dsl::*};

static NOT_FOUND_CODE: u8 = 1;
static TMI_ERROR_CODE: u8 = 2;
static UNKNOW_ERROR_CODE: u8 = 0;

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
    pub id: u32,
    #[diesel(column_name = subsys_name)]
    pub subsys_name: String,
    #[diesel(column_name = url)]
    pub url: String,
    #[diesel(column_name = is_enable)]
    pub is_enable: u8,
    #[diesel(column_name = relate_service)]
    pub relate_service: Option<u32>,
    #[diesel(column_name = update_time)]
    pub update_time: Option<chrono::NaiveDateTime>,
    #[diesel(column_name = token)]
    pub token: String,
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = subsystem_table)]
pub struct SubsysInfo {
    pub id: Option<u32>,
    pub subsys_name: Option<String>,
    pub url: Option<String>,
    pub is_enable: Option<u8>,
    pub relate_service: Option<u32>,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub token: Option<String>,
}

impl SubsysInfo {
    fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        let subsys_info = SubsysInfo {
            id: map.get("id").and_then(|v| v.as_u64().map(|v| v as u32)),
            subsys_name: map
                .get("subsys_name")
                .and_then(|v| v.as_str().map(|v| v.to_string())),
            url: map
                .get("url")
                .and_then(|v| v.as_str().map(|v| v.to_string())),
            is_enable: map
                .get("is_enable")
                .and_then(|v| v.as_u64().map(|v| v as u8)),
            relate_service: map
                .get("relate_service")
                .and_then(|v| v.as_u64().map(|v| v as u32)),
            update_time: Some(Local::now().naive_local()),
            token: map
                .get("token")
                .and_then(|v| v.as_str().map(|v| v.to_string())),
        };

        Ok(subsys_info)
    }
}

// query implement
impl SubsysModel {
    /// get all enable services
    pub fn get_all_enable(conn: &mut MysqlConnection) -> Result<Vec<Self>, (u8, String)> {
        match subsystem_table
            .filter(is_enable.eq(1))
            .select(SubsysModel::as_select())
            .get_results::<SubsysModel>(conn)
        {
            Ok(subsystem_table_data) => Ok(subsystem_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    pub fn get_meta_by_id(
        subsys_id: u32,
        conn: &mut MysqlConnection,
    ) -> Result<Self, (u8, String)> {
        match subsystem_table
            .find(subsys_id)
            .select(SubsysModel::as_select())
            .get_result::<SubsysModel>(conn)
        {
            Ok(subsys) => Ok(subsys),
            Err(NotFound) => Err((
                NOT_FOUND_CODE,
                format!("can NOT find subsystem id: {}.", &subsys_id),
            )),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    pub fn get_enable_by_name(
        name: String,
        conn: &mut MysqlConnection,
    ) -> Result<Self, (u8, String)> {
        match subsystem_table
            .filter(is_enable.eq(1))
            .filter(subsys_name.eq(&name))
            .select(SubsysModel::as_select())
            .get_result::<SubsysModel>(conn)
        {
            Ok(subsys_info) => Ok(subsys_info),
            Err(NotFound) => Err((
                NOT_FOUND_CODE,
                format!("can NOT find subsystem: {}.", &name),
            )),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    /// get all services
    pub fn get_all_with_filter(
        filter: Map<String, Value>,
        conn: &mut MysqlConnection,
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
                    if let Ok(value) = q_v.as_str().unwrap().parse::<u8>() {
                        query = query.filter(is_enable.eq(value));
                    }
                }
                "url" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{value}%");
                        query = query.filter(url.like(pattern));
                    }
                }
                "relate_service" => {
                    if let Ok(value) = q_v.as_str().unwrap().parse::<u32>() {
                        query = query.filter(relate_service.eq(value));
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
        subsys_info: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        let new_subsys = match SubsysInfo::from_map(subsys_info) {
            Ok(subsys) => subsys,
            Err(e) => return Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        };

        let this_subsys_name = new_subsys.subsys_name.clone().unwrap();

        match diesel::insert_into(subsystem_table)
            .values(new_subsys)
            .execute(conn)
        {
            Ok(num_of_change) => Ok(format!(
                "Subsystem meta data {this_subsys_name} created. line: {num_of_change}"
            )),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    pub fn update_meta_by_id(
        subsys_info: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        let update_subsys = match SubsysInfo::from_map(subsys_info) {
            Ok(subsys) => subsys,
            Err(e) => return Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        };

        let this_subsys_id = update_subsys.id.unwrap();

        match diesel::update(subsystem_table.find(this_subsys_id))
            .set(update_subsys)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((NOT_FOUND_CODE, format!("id: {this_subsys_id} not found"))),
                1 => Ok(format!(
                    "{this_subsys_id}'s data updated. lines: {num_of_eff}"
                )),
                _ => Err((
                    TMI_ERROR_CODE,
                    format!("id: {this_subsys_id} Too much info"),
                )),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete_meta_by_id(
        subsys_id: u32,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::delete(subsystem_table.find(subsys_id)).execute(conn) {
            Ok(num_of_eff) => Ok(format!(
                "{subsys_id}'s data deleted. lines: {num_of_eff}"
            )),
            Err(NotFound) => Err((NOT_FOUND_CODE, format!("id: {subsys_id} not found"))),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
