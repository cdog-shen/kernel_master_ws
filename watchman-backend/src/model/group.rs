use chrono::{self, Local};
use diesel::{prelude::*, result::Error::NotFound};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::group_table::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = group_table)]
pub struct GroupModel {
    pub id: i32,
    #[diesel(column_name =  group_name)]
    pub group_name: String,
    #[diesel(column_name = is_enable)]
    pub is_enable: bool,
    #[diesel(column_name = update_time)]
    pub update_time: chrono::NaiveDateTime,
    #[diesel(column_name = user_ids)]
    pub user_ids: serde_json::Value,
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = group_table)]
pub struct GroupInfo {
    pub id: Option<i32>,
    pub group_name: Option<String>,
    pub is_enable: Option<bool>,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub user_ids: Option<serde_json::Value>,
}

impl GroupInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(GroupInfo {
            id: map.get("id").and_then(|v| v.as_i64().map(|i| i as i32)),

            group_name: map
                .get("group_name")
                .and_then(|v| v.as_str().map(|s| s.to_string())),

            is_enable: map.get("is_enable").and_then(|v| v.as_bool()),

            user_ids: map.get("user_ids").cloned(),

            update_time: Some(Local::now().naive_local()),
        })
    }
}

// query implement
impl GroupModel {
    /// get all enable group
    pub fn get_all_enable(conn: &mut PgConnection) -> Result<Vec<Self>, (u8, String)> {
        match group_table
            .filter(is_enable.eq(true))
            .select(GroupModel::as_select())
            .get_results::<GroupModel>(conn)
        {
            Ok(group_table_data) => Ok(group_table_data),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    /// get all of user
    /// need by middleware
    pub fn get_groups_by_uid(uid: i32, conn: &mut PgConnection) -> Result<Vec<Self>, (u8, String)> {
        match group_table
            .filter(is_enable.eq(true))
            .select(GroupModel::as_select())
            .get_results::<GroupModel>(conn)
        {
            Ok(group_table_data) => {
                let result_gids: Vec<GroupModel> = group_table_data
                    .into_iter()
                    .filter_map(|group_info| {
                        if let serde_json::Value::Array(array) = &group_info.user_ids {
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
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, (u8, String)> {
        let mut query = group_table.into_boxed().select(GroupModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "group_name" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(group_name.eq(value));
                    }
                }
                "is_enable" => {
                    if let Some(value) = q_v.as_bool() {
                        query = query.filter(is_enable.eq(value));
                    }
                }
                "user_id" => {
                    if let Some(value) = q_v.as_i64() {
                        query = query.filter(user_ids.contains(serde_json::json!(value)));
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
        group_info: &GroupInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::insert_into(group_table)
            .values(group_info)
            .execute(conn)
        {
            Ok(num_of_change) => Ok(num_of_change),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    pub fn update_group_by_id(
        group_info: &GroupInfo,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(group_table.filter(id.eq(group_info.id.unwrap())))
            .set(group_info)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((
                    BAD_REQUEST_CODE,
                    format!("id: {} not found", group_info.id.unwrap()),
                )),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete_group_by_id(_id: i32, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(group_table.find(_id)).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(NotFound) => Err((BAD_REQUEST_CODE, format!("id: {_id} not found"))),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
