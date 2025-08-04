use chrono::{self, Local};
use diesel::{
    Insertable, PgConnection, Queryable, Selectable, prelude::*, result::Error::NotFound,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::service_table::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

/// The structure of the Service stored in the database.
/// - service_name
///
///     Service name (String)
///
/// - service_point
///
///     Service route (route String)
///
/// - is_enable
///
///    is this service enable (tinyint 1/0)
///
/// - create_time
///
///     create time (datetime %Y-%m-%d %H:%M:%S)
///
#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = service_table)]
pub struct ServiceModel {
    pub id: i32,
    #[diesel(column_name = service_name)]
    pub service_name: String,
    #[diesel(column_name = nick_name)]
    pub nick_name: String,
    #[diesel(column_name = service_point)]
    pub service_point: String,
    #[diesel(column_name = is_enable)]
    pub is_enable: bool,
    #[diesel(column_name = update_time)]
    pub update_time: chrono::NaiveDateTime,
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = service_table)]
pub struct ServiceInfo {
    pub id: Option<i32>,
    pub service_name: Option<String>,
    pub nick_name: Option<String>,
    pub service_point: Option<String>,
    pub is_enable: Option<bool>,
    pub update_time: Option<chrono::NaiveDateTime>,
}

impl ServiceInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(ServiceInfo {
            id: map.get("id").and_then(|v| v.as_i64().map(|i| i as i32)),

            service_name: map
                .get("service_name")
                .and_then(|v| v.as_str().map(|v| v.to_string())),

            nick_name: map
                .get("nick_name")
                .and_then(|v| v.as_str().map(|v| v.to_string())),

            service_point: map
                .get("service_point")
                .and_then(|v| v.as_str().map(|v| v.to_string())),

            is_enable: map.get("is_enable").and_then(|v| v.as_bool()),

            update_time: Some(Local::now().naive_local()),
        })
    }
}

// query implement
impl ServiceModel {
    /// get all enable services
    pub fn get_all_enable(conn: &mut PgConnection) -> Result<Vec<Self>, (u8, String)> {
        match service_table
            .filter(is_enable.eq(true))
            .select(ServiceModel::as_select())
            .get_results::<ServiceModel>(conn)
        {
            Ok(service_table_data) => Ok(service_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    /// get services by id
    pub fn get_services_by_id(
        sid_list: &Vec<i32>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, (u8, String)> {
        match service_table
            .filter(is_enable.eq(true))
            .filter(id.eq_any(sid_list))
            .select(ServiceModel::as_select())
            .get_results::<ServiceModel>(conn)
        {
            Ok(service_table_data) => Ok(service_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    /// get all id by service route
    /// need by middle ware
    pub fn get_sids_by_route(
        route: &str,
        conn: &mut PgConnection,
    ) -> Result<Vec<i32>, (u8, String)> {
        let mut stash_index: Vec<usize> = route
            .chars()
            .enumerate()
            .filter(|&(_, c)| c == '/')
            .map(|(i, _)| i)
            .collect();

        stash_index.push(route.len());

        for index in stash_index {
            let like_pattern = format!("%{}%", &route[..index]);
            // println!("{:?}", like_pattern.len());

            if like_pattern.len() <= 6 {
                continue;
            }

            match service_table
                .filter(is_enable.eq(true))
                .filter(service_point.like(like_pattern))
                .select(id)
                .get_results::<i32>(conn)
            {
                Ok(sids) => match sids.len() {
                    0 => continue,
                    _ => return Ok(sids),
                },
                Err(e) => return Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
            }
        }
        Err((BAD_REQUEST_CODE, "No matching permissions.".to_string()))
    }

    /// get all services
    pub fn get_all_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, (u8, String)> {
        let mut query = service_table.into_boxed().select(ServiceModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "service_name" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(service_name.eq(value));
                    }
                }
                "is_enable" => {
                    if let Some(value) = q_v.as_bool() {
                        query = query.filter(is_enable.eq(value));
                    }
                }
                "service_point" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{value}%");
                        query = query.filter(service_point.like(pattern));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<ServiceModel>(conn) {
            Ok(service_table_data) => Ok(service_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }
}

// update implement
impl ServiceModel {
    pub fn new_service(
        service_info: &ServiceInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::insert_into(service_table)
            .values(service_info)
            .execute(conn)
        {
            Ok(num_of_change) => Ok(num_of_change),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    pub fn update_service_by_id(
        service_info: &ServiceInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(service_table.filter(id.eq(service_info.id.unwrap())))
            .set(service_info)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((
                    BAD_REQUEST_CODE,
                    format!("id: {} not found", service_info.id.unwrap()),
                )),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete_service_by_id(_id: i32, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(service_table.find(_id)).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(NotFound) => Err((BAD_REQUEST_CODE, format!("id: {_id} not found"))),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
