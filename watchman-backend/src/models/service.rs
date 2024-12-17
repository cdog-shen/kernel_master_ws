use chrono::{self, Local};
use diesel::{
    prelude::*, result::Error::NotFound, Insertable, MysqlConnection, Queryable, Selectable,
};
use serde::{Deserialize, Serialize};

use crate::models::schema::service_table::{self, dsl::*};

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
#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = service_table)]
pub struct ServiceModel {
    pub id: u32,
    #[diesel(column_name = service_name)]
    pub service_name: String,
    #[diesel(column_name = service_point)]
    pub service_point: String,
    #[diesel(column_name = is_enable)]
    pub is_enable: u8,
    #[diesel(column_name = create_time)]
    pub create_time: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = service_table)]
pub struct ServiceInputStream {
    pub id: Option<u32>,
    pub service_name: Option<String>,
    pub service_point: Option<String>,
    pub is_enable: Option<u8>,
    pub create_time: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceOutputStream {
    pub id: Option<u32>,
    pub service_name: Option<String>,
    pub service_point: Option<String>,
    pub is_enable: Option<u8>,
    pub create_time: Option<chrono::NaiveDateTime>,
}

fn map_model_to_output_stream(service_info: ServiceModel) -> ServiceOutputStream {
    ServiceOutputStream {
        id: Some(service_info.id),
        service_name: Some(service_info.service_name),
        service_point: Some(service_info.service_point),
        is_enable: Some(service_info.is_enable),
        create_time: service_info.create_time,
    }
}

// query implement
impl ServiceModel {
    /// get all enable services
    pub fn get_all_enable(
        conn: &mut MysqlConnection,
    ) -> Result<Vec<ServiceOutputStream>, (u8, String)> {
        match service_table
            .filter(is_enable.eq(1))
            .select(ServiceModel::as_select())
            .get_results::<ServiceModel>(conn)
        {
            Ok(service_table_data) => {
                let service_output_stream_data: Vec<ServiceOutputStream> = service_table_data
                    .into_iter()
                    .map(|service_info| map_model_to_output_stream(service_info))
                    .collect();
                Ok(service_output_stream_data)
            }
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e.to_string()),
            )),
        }
    }

    /// get services by id
    pub fn get_services_by_id(
        sid_list: Vec<u32>,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<ServiceOutputStream>, (u8, String)> {
        match service_table
            .filter(is_enable.eq(1))
            .filter(id.eq_any(sid_list))
            .select(ServiceModel::as_select())
            .get_results::<ServiceModel>(conn)
        {
            Ok(service_table_data) => {
                let service_output_stream_data: Vec<ServiceOutputStream> = service_table_data
                    .into_iter()
                    .map(|service_info| map_model_to_output_stream(service_info))
                    .collect();
                Ok(service_output_stream_data)
            }
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e.to_string()),
            )),
        }
    }

    /// get all id by service route
    /// need by middle ware
    pub fn get_sids_by_route(
        route: &String,
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
                Err(e) => {
                    return Err((
                        UNKNOW_ERROR_CODE,
                        format!("Unknow Error: {}.", e.to_string()),
                    ))
                }
            }
        }
        Err((NOT_FOUND_CODE, format!("No matching permissions.")))
    }

    /// get all services
    pub fn get_all_with_filter(
        filter: serde_json::Map<String, serde_json::Value>,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<ServiceOutputStream>, (u8, String)> {
        let mut query = service_table.into_boxed().select(ServiceModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "service_name" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(service_name.eq(value));
                    }
                }
                "is_enable" => {
                    println!("{}", q_v.as_str().unwrap().parse::<u8>().is_ok());
                    if let Ok(value) = q_v.as_str().unwrap().parse::<u8>() {
                        query = query.filter(is_enable.eq(value));
                    }
                }
                "service_point" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{}%", value);
                        query = query.filter(service_point.like(pattern));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<ServiceModel>(conn) {
            Ok(service_table_data) => {
                let service_output_stream_data: Vec<ServiceOutputStream> = service_table_data
                    .into_iter()
                    .map(|service_info| map_model_to_output_stream(service_info))
                    .collect();
                Ok(service_output_stream_data)
            }
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e.to_string()),
            )),
        }
    }
}

// update implement
impl ServiceModel {
    pub fn new_service(
        service_info: &ServiceInputStream,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::insert_into(service_table)
            .values((
                service_name.eq(service_info.service_name.clone().unwrap()),
                service_point.eq(service_info.service_point.clone().unwrap()),
                is_enable.eq(service_info.is_enable.unwrap_or(0)),
                create_time.eq(Local::now().naive_local()),
            ))
            .execute(conn)
        {
            Ok(num_of_change) => Ok(format!(
                "Service {} created. line: {}",
                &service_info.service_name.clone().unwrap(),
                num_of_change
            )),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    pub fn update_service_by_id(
        service_info: &ServiceInputStream,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::update(service_table.find(service_info.id.unwrap()))
            .set(service_info)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((
                    NOT_FOUND_CODE,
                    format!("id: {} not found", service_info.id.unwrap()),
                )),
                1 => Ok(format!(
                    "{}'s data updated. lines: {}",
                    service_info.id.unwrap(),
                    num_of_eff
                )),
                _ => Err((
                    TMI_ERROR_CODE,
                    format!("id: {} Too much info", service_info.id.unwrap()),
                )),
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
                "{}'s data deleted. lines: {}",
                service_id, num_of_eff
            )),
            Err(NotFound) => Err((NOT_FOUND_CODE, format!("id: {} not found", service_id))),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
