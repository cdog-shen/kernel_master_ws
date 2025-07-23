use chrono::{self, Local};
use diesel::{prelude::*, result::Error::NotFound};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::group_table::{self, dsl::*};

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
#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
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

impl GroupInfo {
    fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(GroupInfo {
            id: map.get("id").and_then(|v| v.as_u64().map(|v| v as u32)),
            name: map
                .get("name")
                .and_then(|v| v.as_str().map(|v| v.to_string())),
            is_enable: map
                .get("is_enable")
                .and_then(|v| v.as_u64().map(|v| v as u8)),
            date_update: Some(Local::now().naive_local()),
            user_ids: map
                .get("user_ids")
                .and_then(|v| v.as_str().map(|v| v.to_string())),
        })
    }
}

// query implement
impl GroupModel {
    /// get all enable group
    pub fn get_all_enable(conn: &mut MysqlConnection) -> Result<Vec<Self>, (u8, String)> {
        match group_table
            .filter(is_enable.eq(1))
            .select(GroupModel::as_select())
            .get_results::<GroupModel>(conn)
        {
            Ok(group_table_data) => Ok(group_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    /// get all of user
    /// need by middleware
    pub fn get_groups_by_uid(
        uid: u32,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<Self>, (u8, String)> {
        match group_table
            .filter(is_enable.eq(1))
            .select(GroupModel::as_select())
            .get_results::<GroupModel>(conn)
        {
            Ok(group_table_data) => {
                let result_gids: Vec<GroupModel> = group_table_data
                    .into_iter()
                    .filter_map(|group_info| {
                        if let Ok(serde_json::Value::Array(array)) =
                            serde_json::from_str(&group_info.user_ids)
                        {
                            if array.iter().any(|item| {
                                if let Some(i) = item.as_u64() {
                                    i == uid as u64
                                } else {
                                    false
                                }
                            }) {
                                Some(group_info)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();

                Ok(result_gids)
            }
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    /// get all group
    pub fn get_all_with_filter(
        filter: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<Self>, (u8, String)> {
        let mut query = group_table.into_boxed().select(GroupModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "name" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(name.eq(value));
                    }
                }
                "is_enable" => {
                    if let Ok(value) = q_v.as_str().unwrap().parse::<u8>() {
                        query = query.filter(is_enable.eq(value));
                    }
                }
                "user_ids" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern1 = format!("%{value},%");
                        let pattern2 = format!("%{value}]%");
                        query = query
                            .filter(user_ids.like(pattern1))
                            .or_filter(user_ids.like(pattern2));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<GroupModel>(conn) {
            Ok(group_table_data) => Ok(group_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }
}

// update implement
impl GroupModel {
    pub fn new_group(
        group_info: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        let new_group = match GroupInfo::from_map(group_info) {
            Ok(group) => group,
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };

        let this_group_name = new_group.name.clone().unwrap();

        match diesel::insert_into(group_table)
            .values(new_group)
            .execute(conn)
        {
            Ok(num_of_change) => Ok(format!(
                "Group {this_group_name} created. line: {num_of_change}"
            )),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    pub fn update_group_by_id(
        group_info: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        let update_group = match GroupInfo::from_map(group_info) {
            Ok(group) => group,
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };

        let this_group_id = update_group.id.unwrap();

        match diesel::update(group_table.find(this_group_id))
            .set(update_group)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((NOT_FOUND_CODE, format!("id: {this_group_id} not found"))),
                1 => Ok(format!(
                    "{this_group_id}'s data updated. lines: {num_of_eff}"
                )),
                _ => Err((
                    TMI_ERROR_CODE,
                    format!("id: {this_group_id} Too much info"),
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
                "{group_id}'s data deleted. lines: {num_of_eff}"
            )),
            Err(NotFound) => Err((NOT_FOUND_CODE, format!("id: {group_id} not found"))),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
