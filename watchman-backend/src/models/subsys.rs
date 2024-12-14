use chrono::{self, Local};
use diesel::{
    prelude::*, result::Error::NotFound, Insertable, MysqlConnection, Queryable, Selectable,
};
use serde::{Deserialize, Serialize};

use crate::models::schema::subsystem_table::{self, dsl::*};

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
#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable)]
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
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = subsystem_table)]
pub struct SubsysInputStream {
    pub id: Option<u32>,
    pub subsys_name: Option<String>,
    pub url: Option<String>,
    pub is_enable: Option<u8>,
    pub relate_service: Option<u32>,
    pub update_time: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubsysOutputStream {
    pub id: Option<u32>,
    pub subsys_name: Option<String>,
    pub url: Option<String>,
    pub is_enable: Option<u8>,
    pub relate_service: Option<u32>,
    pub update_time: Option<chrono::NaiveDateTime>,
}

fn map_model_to_output_stream(subsys_info: SubsysModel) -> SubsysOutputStream {
    SubsysOutputStream {
        id: Some(subsys_info.id),
        subsys_name: Some(subsys_info.subsys_name),
        url: Some(subsys_info.url),
        is_enable: Some(subsys_info.is_enable),
        relate_service: subsys_info.relate_service,
        update_time: subsys_info.update_time,
    }
}

// query implement
impl SubsysModel {
    /// get all services
    pub fn get_all(conn: &mut MysqlConnection) -> Result<Vec<SubsysOutputStream>, (u8, String)> {
        match subsystem_table
            .select(SubsysModel::as_select())
            .get_results::<SubsysModel>(conn)
        {
            Ok(subsystem_table_data) => {
                let group_output_stream_data: Vec<SubsysOutputStream> = subsystem_table_data
                    .into_iter()
                    .map(|subsys_info| map_model_to_output_stream(subsys_info))
                    .collect();
                Ok(group_output_stream_data)
            }
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e.to_string()),
            )),
        }
    }

    /// get all enable services
    pub fn get_all_enable(
        conn: &mut MysqlConnection,
    ) -> Result<Vec<SubsysOutputStream>, (u8, String)> {
        match subsystem_table
            .filter(is_enable.eq(1))
            .select(SubsysModel::as_select())
            .get_results::<SubsysModel>(conn)
        {
            Ok(subsystem_table_data) => {
                let service_output_stream_data: Vec<SubsysOutputStream> = subsystem_table_data
                    .into_iter()
                    .map(|subsys_info| map_model_to_output_stream(subsys_info))
                    .collect();
                Ok(service_output_stream_data)
            }
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e.to_string()),
            )),
        }
    }

    pub fn get_enable_by_name(
        name: String,
        conn: &mut MysqlConnection,
    ) -> Result<SubsysOutputStream, (u8, String)> {
        match subsystem_table
            .filter(is_enable.eq(1))
            .filter(subsys_name.eq(&name))
            .select(SubsysModel::as_select())
            .get_result::<SubsysModel>(conn)
        {
            Ok(subsys_info) => Ok(map_model_to_output_stream(subsys_info)),
            Err(NotFound) => Err((
                NOT_FOUND_CODE,
                format!("can NOT find subsystem: {}.", &name),
            )),
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e.to_string()),
            )),
        }
    }
}

// update implement
impl SubsysModel {
    pub fn new_meta(
        subsys_info: &SubsysInputStream,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::insert_into(subsystem_table)
            .values((
                subsys_name.eq(subsys_info.subsys_name.clone().unwrap()),
                url.eq(subsys_info.url.clone().unwrap()),
                is_enable.eq(subsys_info.is_enable.unwrap_or(0)),
                relate_service.eq(subsys_info.relate_service.unwrap_or(0)),
                update_time.eq(Local::now().naive_local()),
            ))
            .execute(conn)
        {
            Ok(num_of_change) => Ok(format!(
                "Subsystem meta data {} created. line: {}",
                &subsys_info.subsys_name.clone().unwrap(),
                num_of_change
            )),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    pub fn update_meta_by_id(
        subsys_info: &SubsysInputStream,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::update(subsystem_table.find(subsys_info.id.unwrap()))
            .set(subsys_info)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((
                    NOT_FOUND_CODE,
                    format!("id: {} not found", subsys_info.id.unwrap()),
                )),
                1 => Ok(format!(
                    "{}'s data updated. lines: {}",
                    subsys_info.id.unwrap(),
                    num_of_eff
                )),
                _ => Err((
                    TMI_ERROR_CODE,
                    format!("id: {} Too much info", subsys_info.id.unwrap()),
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
                "{}'s data deleted. lines: {}",
                subsys_id, num_of_eff
            )),
            Err(NotFound) => Err((NOT_FOUND_CODE, format!("id: {} not found", subsys_id))),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
