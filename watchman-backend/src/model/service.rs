use chrono::{self, Local};
use diesel::{
    prelude::*, result::Error::NotFound, Insertable, MysqlConnection, Queryable, Selectable,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::service_table::{self, dsl::*};

static NOT_FOUND_CODE: u8 = 1;
static TMI_ERROR_CODE: u8 = 2;
static UNKNOW_ERROR_CODE: u8 = 0;

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
    pub id: u32,
    #[diesel(column_name = service_name)]
    pub service_name: String,
    #[diesel(column_name = nick_name)]
    pub nick_name: String,
    #[diesel(column_name = service_point)]
    pub service_point: String,
    #[diesel(column_name = is_enable)]
    pub is_enable: u8,
    #[diesel(column_name = create_time)]
    pub create_time: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = service_table)]
pub struct ServiceInfo {
    pub id: Option<u32>,
    pub service_name: Option<String>,
    pub nick_name: Option<String>,
    pub service_point: Option<String>,
    pub is_enable: Option<u8>,
    pub create_time: Option<chrono::NaiveDateTime>,
}

impl ServiceInfo {
    fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(ServiceInfo {
            id: map.get("id").and_then(|v| v.as_u64().map(|v| v as u32)),
            service_name: map
                .get("service_name")
                .and_then(|v| v.as_str().map(|v| v.to_string())),
            nick_name: map
                .get("nick_name")
                .and_then(|v| v.as_str().map(|v| v.to_string())),
            service_point: map
                .get("service_point")
                .and_then(|v| v.as_str().map(|v| v.to_string())),
            is_enable: map
                .get("is_enable")
                .and_then(|v| v.as_u64().map(|v| v as u8)),
            create_time: Some(Local::now().naive_local()),
        })
    }
}

// query implement
impl ServiceModel {
    /// get all enable services
    pub fn get_all_enable(conn: &mut MysqlConnection) -> Result<Vec<Self>, (u8, String)> {
        match service_table
            .filter(is_enable.eq(1))
            .select(ServiceModel::as_select())
            .get_results::<ServiceModel>(conn)
        {
            Ok(service_table_data) => Ok(service_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    /// get services by id
    pub fn get_services_by_id(
        sid_list: Vec<u32>,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<Self>, (u8, String)> {
        match service_table
            .filter(is_enable.eq(1))
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
        conn: &mut MysqlConnection,
    ) -> Result<Vec<u32>, (u8, String)> {
        let stash_index: Vec<usize> = route
            .chars()
            .enumerate()
            .filter(|&(_, c)| c == '/')
            .map(|(i, _)| i)
            .collect();
        for index in stash_index {
            let like_pattern = format!("%{}%", &route[..index]);
            // println!("{:?}", like_pattern.len());

            if like_pattern.len() <= 6 {
                continue;
            }

            match service_table
                .filter(is_enable.eq(1))
                .filter(service_point.like(like_pattern)) // 使用LIKE进行模糊匹配
                .select(id)
                .get_results::<u32>(conn)
            {
                Ok(sids) => match sids.len() {
                    0 => continue,
                    _ => return Ok(sids),
                },
                Err(e) => return Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
            }
        }
        Err((NOT_FOUND_CODE, "No matching permissions.".to_string()))
    }

    /// get all services
    pub fn get_all_with_filter(
        filter: Map<String, Value>,
        conn: &mut MysqlConnection,
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
                    if let Ok(value) = q_v.as_str().unwrap().parse::<u8>() {
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
        service_info: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        let new_service = match ServiceInfo::from_map(service_info) {
            Ok(service) => service,
            Err(e) => return Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        };

        let this_service_name = new_service.service_name.clone().unwrap();

        match diesel::insert_into(service_table)
            .values(new_service)
            .execute(conn)
        {
            Ok(num_of_change) => Ok(format!(
                "Service {this_service_name} created. line: {num_of_change}"
            )),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    pub fn update_service_by_id(
        service_info: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        let update_service = match ServiceInfo::from_map(service_info) {
            Ok(service) => service,
            Err(e) => return Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        };

        let this_id = update_service.id.unwrap();

        match diesel::update(service_table.find(this_id))
            .set(update_service)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((NOT_FOUND_CODE, format!("id: {this_id} not found"))),
                1 => Ok(format!("{this_id}'s data updated. lines: {num_of_eff}")),
                _ => Err((TMI_ERROR_CODE, format!("id: {this_id} Too much info"))),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete_service_by_id(
        service_id: u32,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::delete(service_table.find(service_id)).execute(conn) {
            Ok(num_of_eff) => Ok(format!(
                "{service_id}'s data deleted. lines: {num_of_eff}"
            )),
            Err(NotFound) => Err((NOT_FOUND_CODE, format!("id: {service_id} not found"))),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
