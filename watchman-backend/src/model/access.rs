use chrono::{self, Local};
use diesel::{
    prelude::*, result::Error::NotFound, Insertable, MysqlConnection, Queryable, Selectable,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::access_table::{self, dsl::*};

static NOT_FOUND_CODE: u8 = 1;
static TMI_ERROR_CODE: u8 = 2;
static UNKNOW_ERROR_CODE: u8 = 0;

/// The structure of the Access stored in the database.
/// - access_id
///
///     Access Id (u32)
///
/// - service_id
///
///     service id from service table (u32)
///
///  - group_id
///
///     group id from group table (u32)
///
///  - group access
///
///     group access for is service (u8 tinyint)
///
/// - is_enable
///
///    is this group enable (tinyint 1/0)
///
/// - create_time
///
///     create time (datetime %Y-%m-%d %H:%M:%S)
///
#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = access_table)]
pub struct AccessModel {
    pub id: u32,
    #[diesel(column_name = service_id)]
    pub service_id: u32,
    #[diesel(column_name = group_id)]
    pub group_id: u32,
    #[diesel(column_name = group_access)]
    pub group_access: u8,
    #[diesel(column_name = is_enable)]
    pub is_enable: u8,
    #[diesel(column_name = update_time)]
    pub update_time: Option<chrono::NaiveDateTime>,
    #[diesel(column_name = comment)]
    pub comment: Option<String>,
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = access_table)]
pub struct AccessInfo {
    pub id: Option<u32>,
    pub service_id: Option<u32>,
    pub group_id: Option<u32>,
    pub group_access: Option<u8>,
    pub is_enable: Option<u8>,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub comment: Option<String>,
}

impl AccessInfo {
    fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(AccessInfo {
            id: map.get("id").and_then(|v| v.as_u64().map(|v| v as u32)),
            service_id: map
                .get("service_id")
                .and_then(|v| v.as_u64().map(|v| v as u32)),
            group_id: map
                .get("group_id")
                .and_then(|v| v.as_u64().map(|v| v as u32)),
            group_access: map
                .get("group_access")
                .and_then(|v| v.as_u64().map(|v| v as u8)),
            is_enable: map
                .get("is_enable")
                .and_then(|v| v.as_u64().map(|v| v as u8)),
            update_time: Some(Local::now().naive_local()),
            comment: map
                .get("comment")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        })
    }
}

// query implement
impl AccessModel {
    /// get all enable access
    pub fn get_all_enable(conn: &mut MysqlConnection) -> Result<Vec<Self>, (u8, String)> {
        match access_table
            .filter(is_enable.eq(1))
            .select(AccessModel::as_select())
            .get_results::<AccessModel>(conn)
        {
            Ok(access_table_data) => Ok(access_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {}.", e))),
        }
    }

    pub fn get_access_by_gids(
        gid_list: Vec<u32>,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<Self>, (u8, String)> {
        match access_table
            .filter(is_enable.eq(1))
            .filter(group_id.eq_any(gid_list))
            .select(AccessModel::as_select())
            .get_results::<AccessModel>(conn)
        {
            Ok(access_table_data) => Ok(access_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {}.", e))),
        }
    }

    pub fn get_access_by_sids(
        sid_list: Vec<u32>,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<Self>, (u8, String)> {
        match access_table
            .filter(is_enable.eq(1))
            .filter(service_id.eq_any(sid_list))
            .select(AccessModel::as_select())
            .get_results::<AccessModel>(conn)
        {
            Ok(access_table_data) => Ok(access_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {}.", e))),
        }
    }

    /// get user's max access
    /// need by middleware
    pub fn get_max_permission(
        gid_list: Vec<u32>,
        sid_list: Vec<u32>,
        conn: &mut MysqlConnection,
    ) -> Result<u8, (u8, String)> {
        match access_table
            .filter(is_enable.eq(1))
            .filter(group_id.eq_any(gid_list))
            .filter(service_id.eq_any(sid_list))
            .select(AccessModel::as_select())
            .get_results::<AccessModel>(conn)
        {
            Ok(access_table_data) => {
                let mut max_access: u8 = 0;
                for access_info in access_table_data.into_iter() {
                    if access_info.group_access > max_access {
                        max_access = access_info.group_access
                    };
                }
                Ok(max_access)
            }
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {}.", e))),
        }
    }

    /// get all access
    pub fn get_all_with_filter(
        filter: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<Self>, (u8, String)> {
        let mut query = access_table.into_boxed().select(AccessModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "service_id" => {
                    if let Ok(value) = q_v.as_str().unwrap().parse::<u32>() {
                        query = query.filter(service_id.eq(value));
                    }
                }
                "group_id" => {
                    if let Ok(value) = q_v.as_str().unwrap().parse::<u32>() {
                        query = query.filter(group_id.eq(value));
                    }
                }
                "group_access" => {
                    if let Ok(value) = q_v.as_str().unwrap().parse::<u8>() {
                        query = query.filter(group_access.eq(value));
                    }
                }
                "is_enable" => {
                    if let Ok(value) = q_v.as_str().unwrap().parse::<u8>() {
                        query = query.filter(is_enable.eq(value));
                    }
                }
                "comment" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{}%", value);
                        query = query.filter(comment.like(pattern));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<AccessModel>(conn) {
            Ok(access_table_data) => Ok(access_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {}.", e))),
        }
    }
}

// update implement
impl AccessModel {
    pub fn new_access(
        access_info: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        let new_access = match AccessInfo::from_map(access_info) {
            Ok(access) => access,
            Err(e) => return Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {}.", e))),
        };

        match diesel::insert_into(access_table)
            .values(new_access)
            .execute(conn)
        {
            Ok(num_of_change) => Ok(format!("Access created. line: {}", num_of_change,)),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    pub fn update_access_by_id(
        access_info: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        let update_access = match AccessInfo::from_map(access_info) {
            Ok(access) => access,
            Err(e) => return Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {}.", e))),
        };

        let this_access_id = update_access.id.clone().unwrap();

        match diesel::update(access_table.find(&this_access_id))
            .set(update_access)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((NOT_FOUND_CODE, format!("id: {} not found", this_access_id))),
                1 => Ok(format!(
                    "{}'s data updated. lines: {}",
                    this_access_id, num_of_eff
                )),
                _ => Err((
                    TMI_ERROR_CODE,
                    format!("id: {} Too much info", this_access_id),
                )),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete_access_by_id(
        access_id: u32,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::delete(access_table.find(access_id)).execute(conn) {
            Ok(num_of_eff) => Ok(format!(
                "{}'s data deleted. lines: {}",
                access_id, num_of_eff
            )),
            Err(NotFound) => Err((NOT_FOUND_CODE, format!("id: {} not found", access_id))),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
