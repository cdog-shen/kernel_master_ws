use chrono::{self, Local};
use diesel::{
    prelude::*, result::Error::NotFound, Insertable, MysqlConnection, Queryable, Selectable,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::models::schema::access_table::{self, dsl::*};

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
#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable)]
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
pub struct AccessInputStream {
    pub id: Option<u32>,
    pub service_id: Option<u32>,
    pub group_id: Option<u32>,
    pub group_access: Option<u8>,
    pub is_enable: Option<u8>,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessOutputStream {
    pub id: Option<u32>,
    pub service_id: Option<u32>,
    pub group_id: Option<u32>,
    pub group_access: Option<u8>,
    pub is_enable: Option<u8>,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub comment: Option<String>,
}

fn map_model_to_output_stream(access_info: AccessModel) -> AccessOutputStream {
    AccessOutputStream {
        id: Some(access_info.id),
        service_id: Some(access_info.service_id),
        group_id: Some(access_info.group_id),
        group_access: Some(access_info.group_access),
        is_enable: Some(access_info.is_enable),
        update_time: access_info.update_time,
        comment: access_info.comment,
    }
}

// query implement
impl AccessModel {
    /// get all enable access
    pub fn get_all_enable(
        conn: &mut MysqlConnection,
    ) -> Result<Vec<AccessOutputStream>, (u8, String)> {
        match access_table
            .filter(is_enable.eq(1))
            .select(AccessModel::as_select())
            .get_results::<AccessModel>(conn)
        {
            Ok(access_table_data) => {
                let group_output_stream_data: Vec<AccessOutputStream> = access_table_data
                    .into_iter()
                    .map(|access_info| map_model_to_output_stream(access_info))
                    .collect();
                Ok(group_output_stream_data)
            }
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e.to_string()),
            )),
        }
    }

    pub fn get_access_by_gids(
        gid_list: Vec<u32>,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<AccessOutputStream>, (u8, String)> {
        match access_table
            .filter(is_enable.eq(1))
            .filter(group_id.eq_any(gid_list))
            .select(AccessModel::as_select())
            .get_results::<AccessModel>(conn)
        {
            Ok(access_table_data) => Ok(access_table_data
                .into_iter()
                .map(|access_info| map_model_to_output_stream(access_info))
                .collect()),
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e.to_string()),
            )),
        }
    }

    pub fn get_access_by_sids(
        sid_list: Vec<u32>,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<AccessOutputStream>, (u8, String)> {
        match access_table
            .filter(is_enable.eq(1))
            .filter(service_id.eq_any(sid_list))
            .select(AccessModel::as_select())
            .get_results::<AccessModel>(conn)
        {
            Ok(access_table_data) => Ok(access_table_data
                .into_iter()
                .map(|access_info| map_model_to_output_stream(access_info))
                .collect()),
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e.to_string()),
            )),
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
                let mut max_access_strea: u8 = 0;
                for access_info in access_table_data
                    .into_iter()
                    .map(|access_info| map_model_to_output_stream(access_info))
                {
                    match access_info.group_access.unwrap_or(0) > max_access_strea {
                        true => max_access_strea = access_info.group_access.unwrap_or(0),
                        false => (),
                    };
                }
                Ok(max_access_strea)
            }
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e.to_string()),
            )),
        }
    }

    /// get all access
    pub fn get_all_with_filter(
        filter: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<AccessOutputStream>, (u8, String)> {
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
            Ok(access_table_data) => {
                let group_output_stream_data: Vec<AccessOutputStream> = access_table_data
                    .into_iter()
                    .map(|access_info| map_model_to_output_stream(access_info))
                    .collect();
                Ok(group_output_stream_data)
            }
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e.to_string()),
            )),
        }
    }
}

// update implement
impl AccessModel {
    pub fn new_access(
        access_info: &AccessInputStream,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::insert_into(access_table)
            .values((
                service_id.eq(access_info.service_id.unwrap()),
                group_id.eq(access_info.group_id.unwrap()),
                group_access.eq(access_info.group_access.unwrap()),
                is_enable.eq(access_info.is_enable.unwrap_or(0)),
                update_time.eq(Local::now().naive_local()),
                comment.eq(access_info
                    .comment
                    .clone()
                    .unwrap_or("none set".to_string())),
            ))
            .execute(conn)
        {
            Ok(num_of_change) => Ok(format!(
                "Access {} created. line: {}",
                &access_info.service_id.clone().unwrap(),
                num_of_change
            )),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    pub fn update_access_by_id(
        access_info: &AccessInputStream,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::update(access_table.find(access_info.id.unwrap()))
            .set(access_info)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((
                    NOT_FOUND_CODE,
                    format!("id: {} not found", access_info.id.unwrap()),
                )),
                1 => Ok(format!(
                    "{}'s data updated. lines: {}",
                    access_info.id.unwrap(),
                    num_of_eff
                )),
                _ => Err((
                    TMI_ERROR_CODE,
                    format!("id: {} Too much info", access_info.id.unwrap()),
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
