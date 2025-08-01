use chrono::{self, Local};
use diesel::{prelude::*, result::Error::NotFound};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::user_table::{self, dsl::*};

static NOT_FOUND_CODE: u8 = 1;
static UNKNOW_ERROR_CODE: u8 = 0;
static TMI_ERROR_CODE: u8 = 2;

/// The structure of the user stored in the database.
#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = user_table)]
pub struct UserModel {
    pub id: i32,
    pub username: String,
    pub passwd: String,
    pub is_enable: bool,
    pub full_name: String,
    pub contact: serde_json::Value,
    pub last_login: chrono::NaiveDateTime,
    pub update_time: chrono::NaiveDateTime,
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = user_table)]
pub struct UserInputStream {
    pub id: Option<i32>,
    pub username: Option<String>,
    pub passwd: Option<String>,
    pub is_enable: Option<bool>,
    pub full_name: Option<String>,
    pub contact: Option<serde_json::Value>,
    pub update_time: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserOutputStream {
    pub id: Option<i32>,
    pub username: Option<String>,
    pub is_enable: Option<bool>,
    pub full_name: Option<String>,
    pub contact: Option<serde_json::Value>,
    pub last_login: Option<chrono::NaiveDateTime>,
    pub update_time: Option<chrono::NaiveDateTime>,
}

impl UserOutputStream {
    fn from_model(user_info: UserModel) -> Result<Self, String> {
        Ok(UserOutputStream {
            id: Some(user_info.id),
            username: Some(user_info.username),
            is_enable: Some(user_info.is_enable),
            full_name: Some(user_info.full_name),
            contact: Some(user_info.contact),
            last_login: Some(user_info.last_login),
            update_time: Some(user_info.update_time),
        })
    }
}

impl UserInputStream {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(UserInputStream {
            id: map.get("id").and_then(|v| v.as_i64().map(|i| i as i32)),

            username: map
                .get("username")
                .and_then(|v| v.as_str().map(|s| s.to_string())),

            passwd: map
                .get("passwd")
                .and_then(|v| v.as_str().map(|s| s.to_string())),

            is_enable: map.get("is_enable").and_then(|v| v.as_bool()),

            full_name: map
                .get("full_name")
                .and_then(|v| v.as_str().map(|s| s.to_string())),

            contact: map.get("contact").cloned(),

            update_time: Some(Local::now().naive_local()),
        })
    }
}

// query implement
impl UserModel {
    /// get user by username
    /// need by middleware
    pub fn get_user_by_username(
        user_name: &str,
        conn: &mut PgConnection,
    ) -> Result<UserOutputStream, (u8, String)> {
        match user_table
            .filter(is_enable.eq(true))
            .filter(username.eq(user_name))
            .select(UserModel::as_select())
            .get_result::<UserModel>(conn)
        {
            Ok(user_model) => match UserOutputStream::from_model(user_model) {
                Ok(the_struct) => Ok(the_struct),
                Err(e) => Err((UNKNOW_ERROR_CODE, e)),
            },
            Err(NotFound) => Err((NOT_FOUND_CODE, format!("NotFound {user_name}."))),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    /// get user by username
    pub fn get_user_by_id(
        uid: i32,
        conn: &mut PgConnection,
    ) -> Result<UserOutputStream, (u8, String)> {
        match user_table
            .find(uid)
            .select(UserModel::as_select())
            .get_result::<UserModel>(conn)
        {
            Ok(user_model) => match UserOutputStream::from_model(user_model) {
                Ok(the_struct) => Ok(the_struct),
                Err(e) => Err((UNKNOW_ERROR_CODE, e)),
            },
            Err(NotFound) => Err((NOT_FOUND_CODE, format!("NotFound {uid}."))),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    /// login query
    pub fn login(
        login_info: &UserInputStream,
        conn: &mut PgConnection,
    ) -> Result<UserOutputStream, (u8, String)> {
        match user_table
            .filter(is_enable.eq(true))
            .filter(username.eq(login_info.username.clone().unwrap()))
            .filter(passwd.eq(login_info.passwd.clone().unwrap()))
            .select(UserModel::as_select())
            .get_result::<UserModel>(conn)
        {
            Ok(user_model) => match UserOutputStream::from_model(user_model) {
                Ok(the_struct) => Ok(the_struct),
                Err(e) => Err((UNKNOW_ERROR_CODE, e)),
            },
            Err(NotFound) => Err((NOT_FOUND_CODE, "Login Failed.".to_string())),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow Error: {e}."))),
        }
    }

    /// get user full data
    pub fn get_user_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<UserOutputStream>, (u8, String)> {
        let mut query = user_table.into_boxed().select(UserModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "username" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(username.eq(value));
                    }
                }
                "is_enable" => {
                    if let Some(value) = q_v.as_bool() {
                        query = query.filter(is_enable.eq(value));
                    }
                }
                "full_name" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{value}%");
                        query = query.filter(full_name.like(pattern));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<UserModel>(conn) {
            Ok(vec_user_info) => Ok(vec_user_info
                .into_iter()
                .map(|user_model| {
                    UserOutputStream::from_model(user_model).unwrap_or_else(|e| UserOutputStream {
                        id: None,
                        username: None,
                        is_enable: None,
                        full_name: Some(e),
                        contact: None,
                        last_login: None,
                        update_time: None,
                    })
                })
                .collect()),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}

// update query
impl UserModel {
    /// creat a user (not enable it)
    pub fn new_user(
        user_info: &UserInputStream,
        conn: &mut PgConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::insert_into(user_table)
            .values(user_info)
            .execute(conn)
        {
            Ok(num_of_change) => Ok(format!(
                "User {} created. line: {num_of_change}",
                user_info.username.clone().unwrap(),
            )),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    /// update an user's data
    pub fn update_user_by_id(
        user_info: &UserInputStream,
        conn: &mut PgConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::update(user_table.filter(id.eq(user_info.id.unwrap())))
            .set(user_info)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((
                    NOT_FOUND_CODE,
                    format!("id: {} not found", user_info.id.clone().unwrap()),
                )),
                1 => Ok(format!(
                    "{}'s data updated. lines: {}",
                    user_info.id.unwrap(),
                    num_of_eff
                )),
                _ => Err((
                    TMI_ERROR_CODE,
                    format!("id: {} Too much info", user_info.id.unwrap()),
                )),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    /// refresh user's last login time
    pub fn update_last_login(
        user_data: &UserInputStream,
        conn: &mut PgConnection,
    ) -> Result<String, (u8, String)> {
        let target = user_table.filter(username.eq(user_data.username.clone().unwrap()));

        let _: Result<i64, (u8, String)> = match target.clone().count().get_result(conn) {
            Ok(c) => match c {
                1 => Ok(1),
                _ => {
                    return Err((
                        NOT_FOUND_CODE,
                        format!(
                            "can NOT find specific user: {}.",
                            &user_data.username.clone().unwrap()
                        ),
                    ));
                }
            },
            Err(e) => return Err((UNKNOW_ERROR_CODE, e.to_string())),
        };

        match diesel::update(target)
            .set(last_login.eq(chrono::Local::now().naive_local()))
            .execute(conn)
        {
            Ok(num_of_change) => Ok(format!(
                "{}'s last login time update. lines: {}",
                &user_data.username.clone().unwrap(),
                num_of_change
            )),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }
}
