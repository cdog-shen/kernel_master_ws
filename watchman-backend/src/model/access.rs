use chrono::{self, Local};
use diesel::{
    Insertable, PgConnection, Queryable, Selectable, prelude::*, result::Error::NotFound,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::access_table::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = access_table)]
pub struct AccessModel {
    pub id: i32,
    #[diesel(column_name = service_id)]
    pub service_id: i32,
    #[diesel(column_name = group_id)]
    pub group_id: i32,
    #[diesel(column_name = group_access)]
    pub group_access: i16,
    #[diesel(column_name = is_enable)]
    pub is_enable: bool,
    #[diesel(column_name = update_time)]
    pub update_time: chrono::NaiveDateTime,
    #[diesel(column_name = comment)]
    pub comment: String,
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = access_table)]
pub struct AccessInfo {
    pub id: Option<i32>,
    pub service_id: Option<i32>,
    pub group_id: Option<i32>,
    pub group_access: Option<i16>,
    pub is_enable: Option<bool>,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub comment: Option<String>,
}

impl AccessInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(AccessInfo {
            id: map.get("id").and_then(|v| v.as_i64().map(|i| i as i32)),

            service_id: map
                .get("service_id")
                .and_then(|v| v.as_i64().map(|i| i as i32)),

            group_id: map
                .get("group_id")
                .and_then(|v| v.as_i64().map(|i| i as i32)),

            group_access: map
                .get("group_access")
                .and_then(|v| v.as_i64().map(|i| i as i16)),

            is_enable: map.get("is_enable").and_then(|v| v.as_bool()),

            comment: map
                .get("comment")
                .and_then(|v| v.as_str().map(|s| s.to_string())),

            update_time: Some(Local::now().naive_local()),
        })
    }
}

// query implement
impl AccessModel {
    /// get all enable access
    pub fn get_all_enable(conn: &mut PgConnection) -> Result<Vec<Self>, (u8, String)> {
        match access_table
            .filter(is_enable.eq(true))
            .select(AccessModel::as_select())
            .get_results::<AccessModel>(conn)
        {
            Ok(access_table_data) => Ok(access_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    pub fn get_access_by_gids(
        gid_list: &Vec<i32>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, (u8, String)> {
        match access_table
            .filter(is_enable.eq(true))
            .filter(group_id.eq_any(gid_list))
            .select(AccessModel::as_select())
            .get_results::<AccessModel>(conn)
        {
            Ok(access_table_data) => Ok(access_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    pub fn get_access_by_sids(
        sid_list: &Vec<i32>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, (u8, String)> {
        match access_table
            .filter(is_enable.eq(true))
            .filter(service_id.eq_any(sid_list))
            .select(AccessModel::as_select())
            .get_results::<AccessModel>(conn)
        {
            Ok(access_table_data) => Ok(access_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    /// get user's max access
    /// need by middleware
    pub fn get_max_permission(
        gid_list: &Vec<i32>,
        sid_list: &Vec<i32>,
        conn: &mut PgConnection,
    ) -> Result<i16, (u8, String)> {
        match access_table
            .filter(is_enable.eq(true))
            .filter(group_id.eq_any(gid_list))
            .filter(service_id.eq_any(sid_list))
            .select(AccessModel::as_select())
            .get_results::<AccessModel>(conn)
        {
            Ok(access_table_data) => {
                let mut max_access: i16 = 0;
                for access_info in access_table_data.into_iter() {
                    if access_info.group_access > max_access {
                        max_access = access_info.group_access
                    };
                }
                Ok(max_access)
            }
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    /// get all access
    pub fn get_all_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, (u8, String)> {
        let mut query = access_table.into_boxed().select(AccessModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "service_id" => {
                    if let Some(value) = q_v.as_i64() {
                        query = query.filter(service_id.eq(value as i32));
                    }
                }
                "group_id" => {
                    if let Some(value) = q_v.as_i64() {
                        query = query.filter(group_id.eq(value as i32));
                    }
                }
                "group_access" => {
                    if let Some(value) = q_v.as_i64() {
                        query = query.filter(group_access.eq(value as i16));
                    }
                }
                "is_enable" => {
                    if let Some(value) = q_v.as_bool() {
                        query = query.filter(is_enable.eq(value));
                    }
                }
                "comment" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{value}%");
                        query = query.filter(comment.like(pattern));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<AccessModel>(conn) {
            Ok(access_table_data) => Ok(access_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }
}

// update implement
impl AccessModel {
    pub fn new_access(
        access_info: &AccessInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::insert_into(access_table)
            .values(access_info)
            .execute(conn)
        {
            Ok(num_of_change) => Ok(num_of_change),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    pub fn update_access_by_id(
        access_info: &AccessInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(access_table.filter(id.eq(access_info.id.unwrap())))
            .set(access_info)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((
                    BAD_REQUEST_CODE,
                    format!("id: {} not found", access_info.id.unwrap()),
                )),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete_access_by_id(_id: i32, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(access_table.find(_id)).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(NotFound) => Err((BAD_REQUEST_CODE, format!("id: {_id} not found"))),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
