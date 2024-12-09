use chrono::{self, Local};
use diesel::{prelude::*, result::Error::NotFound};
use serde::{Deserialize, Serialize};

use crate::models::schema::group_table::{self, dsl::*};

static NOT_FOUND_CODE: u8 = 1;
static TMI_ERROR_CODE: u8 = 2;
static UNKNOW_ERROR_CODE: u8 = 0;

/// The structure of the Group stored in the database.
/// - name
///
///     Group name (String)
///
/// - is_enable
///
///    is this group enable (tinyint 1/0)
///
/// - date_update
///
///     last update time (datetime %Y-%m-%d %H:%M:%S)
///
/// - user_ids
///
///     user's id in this group (json String)
#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = group_table)]
pub struct GroupModel {
    pub id: u32,
    #[diesel(column_name = name)]
    pub name: String,
    #[diesel(column_name = is_enable)]
    pub is_enable: u8,
    #[diesel(column_name = date_update)]
    pub date_update: Option<chrono::NaiveDateTime>,
    #[diesel(column_name = user_ids)]
    pub user_ids: String,
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = group_table)]
pub struct GroupInfo {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub is_enable: Option<u8>,
    pub date_update: Option<chrono::NaiveDateTime>,
    pub user_ids: Option<String>,
}

// query implement
impl GroupModel {
    /// get all group
    pub fn get_all(conn: &mut MysqlConnection) -> Result<Vec<GroupModel>, (u8, String)> {
        match group_table
            .select(GroupModel::as_select())
            .get_results::<GroupModel>(conn)
        {
            Ok(group_table_data) => Ok(group_table_data),
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e.to_string()),
            )),
        }
    }

    /// get all enable group
    pub fn get_all_enable(conn: &mut MysqlConnection) -> Result<GroupModel, (u8, String)> {
        match group_table
            .filter(is_enable.eq(1))
            .select(GroupModel::as_select())
            .get_result::<GroupModel>(conn)
        {
            Ok(group_table_data) => Ok(group_table_data),
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e.to_string()),
            )),
        }
    }
}

// update implement
impl GroupModel {
    pub fn new_group(
        group_info: &GroupInfo,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::insert_into(group_table)
            .values((
                name.eq(group_info.name.clone().unwrap()),
                is_enable.eq(group_info.is_enable.unwrap_or(0)),
                date_update.eq(Local::now().naive_local()),
                user_ids.eq(group_info.user_ids.clone().unwrap_or("[]".to_string())),
            ))
            .execute(conn)
        {
            Ok(num_of_change) => Ok(format!(
                "Group {} created. line: {}",
                &group_info.name.clone().unwrap(),
                num_of_change
            )),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    pub fn update_group_by_id(
        group_info: &GroupInfo,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::update(group_table.find(group_info.id.unwrap()))
            .set(group_info)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((
                    NOT_FOUND_CODE,
                    format!("id: {} not found", group_info.id.unwrap()),
                )),
                1 => Ok(format!(
                    "{}'s data updated. lines: {}",
                    group_info.id.unwrap(),
                    num_of_eff
                )),
                _ => Err((
                    TMI_ERROR_CODE,
                    format!("id: {} Too much info", group_info.id.unwrap()),
                )),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete_group_by_id(
        group_id: u32,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::delete(group_table.find(group_id)).execute(conn) {
            Ok(num_of_eff) => Ok(format!(
                "{}'s data deleted. lines: {}",
                group_id, num_of_eff
            )),
            Err(NotFound) => Err((NOT_FOUND_CODE, format!("id: {} not found", group_id))),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
