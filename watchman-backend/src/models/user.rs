use chrono::{self, Local};
use diesel::{prelude::*, result::Error::NotFound};
use serde::{Deserialize, Serialize};

use crate::models::schema::user_table::{self, dsl::*};

static NOT_FOUND_CODE: u8 = 1;
static UNKNOW_ERROR_CODE: u8 = 0;
static TMI_ERROR_CODE: u8 = 2;

/// The structure of the user stored in the database.
#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = user_table)]
pub struct UserModel {
    pub id: u32,
    pub username: String,
    pub passwd: String,
    pub is_enable: u8,
    pub name: Option<String>,
    pub contact: Option<String>,
    pub date_joined: Option<chrono::NaiveDateTime>,
    pub last_login: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = user_table)]
pub struct UserInputStream {
    pub id: Option<u32>,
    pub username: Option<String>,
    pub passwd: Option<String>,
    pub is_enable: Option<u8>,
    pub name: Option<String>,
    pub contact: Option<String>,
    pub date_joined: Option<chrono::NaiveDateTime>,
    pub last_login: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserOutputStream {
    pub id: Option<u32>,
    pub username: Option<String>,
    pub is_enable: Option<u8>,
    pub name: Option<String>,
    pub contact: Option<serde_json::Value>,
    pub date_joined: Option<chrono::NaiveDateTime>,
    pub last_login: Option<chrono::NaiveDateTime>,
}

fn map_model_to_output_stream(user_info: UserModel) -> UserOutputStream {
    UserOutputStream {
        id: Some(user_info.id),
        username: Some(user_info.username),
        is_enable: Some(user_info.is_enable),
        name: user_info.name,
        contact: serde_json::from_str(&user_info.contact.clone().unwrap_or("".to_string()))
            .unwrap_or(
                serde_json::from_str(
                    format!("[{}]", &user_info.contact.unwrap_or("".to_string())).as_str(),
                )
                .ok(),
            ),
        date_joined: user_info.date_joined,
        last_login: user_info.last_login,
    }
}

// query implement
impl UserModel {
    /// get user by username
    /// need by middleware
    pub fn get_user_by_username(
        user_name: &str,
        conn: &mut MysqlConnection,
    ) -> Result<UserOutputStream, (u8, String)> {
        match user_table
            .filter(is_enable.eq(1))
            .filter(username.eq(user_name))
            .select(UserModel::as_select())
            .get_result::<UserModel>(conn)
        {
            Ok(user_info) => Ok(map_model_to_output_stream(user_info)),
            Err(NotFound) => Err((NOT_FOUND_CODE, format!("NotFound {}.", user_name))),
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e),
            )),
        }
    }

    /// get user by username
    pub fn get_user_by_id(
        uid: u32,
        conn: &mut MysqlConnection,
    ) -> Result<UserOutputStream, (u8, String)> {
        match user_table
            .find(uid)
            .select(UserModel::as_select())
            .get_result::<UserModel>(conn)
        {
            Ok(user_info) => Ok(map_model_to_output_stream(user_info)),
            Err(NotFound) => Err((NOT_FOUND_CODE, format!("NotFound {}.", uid))),
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e),
            )),
        }
    }

    /// login query
    pub fn login(
        user_data: &UserInputStream,
        conn: &mut MysqlConnection,
    ) -> Result<UserOutputStream, (u8, String)> {
        match user_table
            .filter(is_enable.eq(1))
            .filter(username.eq(&user_data.username.clone().unwrap()))
            .filter(passwd.eq(&user_data.passwd.clone().unwrap_or("".to_string())))
            .select(UserModel::as_select())
            .get_result::<UserModel>(conn)
        {
            Ok(user_info) => Ok(map_model_to_output_stream(user_info)),
            Err(NotFound) => Err((
                NOT_FOUND_CODE,
                format!("Login filed: {}.", &user_data.username.clone().unwrap()),
            )),
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e),
            )),
        }
    }

    /// get user full data
    pub fn get_user_info_with_filter(
        filter: serde_json::Map<String, serde_json::Value>,
        conn: &mut MysqlConnection,
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
                    if let Ok(value) = q_v.as_str().unwrap().parse::<u8>() {
                        query = query.filter(is_enable.eq(value));
                    }
                }
                "name" => {
                    if let Some(value) = q_v.as_str() {
                        let pattern = format!("%{}%", value);
                        query = query.filter(name.like(pattern));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<UserModel>(conn) {
            Ok(vec_user_info) => Ok(vec_user_info
                .into_iter()
                .map(map_model_to_output_stream)
                .collect()),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}

// update query
impl UserModel {
    /// creat a user (not enable it)
    pub fn new_user(
        user_data: &UserInputStream,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::insert_into(user_table)
            .values((
                username.eq(&user_data.username.clone().unwrap()),
                passwd.eq(&user_data.passwd.clone().unwrap_or("".to_string())),
                date_joined.eq(Local::now().naive_local()),
            ))
            .execute(conn)
        {
            Ok(num_of_change) => Ok(format!(
                "User {} created. line: {}",
                &user_data.username.clone().unwrap(),
                num_of_change
            )),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    /// update an user's data
    pub fn update_user_by_id(
        user_update: &UserInputStream,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::update(user_table.find(user_update.id.unwrap()))
            .set(user_update)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((
                    NOT_FOUND_CODE,
                    format!("id: {} not found", user_update.id.unwrap()),
                )),
                1 => Ok(format!(
                    "{}'s data updated. lines: {}",
                    user_update.id.unwrap(),
                    num_of_eff
                )),
                _ => Err((
                    TMI_ERROR_CODE,
                    format!("id: {} Too much info", user_update.id.unwrap()),
                )),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    /// refresh user's last login time
    pub fn update_last_login(
        user_data: &UserInputStream,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        let binding = user_data.username.clone().unwrap();
        let target = user_table.filter(username.eq(&binding));

        let _: Result<i64, (u8, String)> = match target.count().get_result(conn) {
            Ok(c) => match c {
                1 => Ok(1),
                _ => {
                    return Err((
                        NOT_FOUND_CODE,
                        format!(
                            "can NOT find specific user: {}.",
                            &user_data.username.clone().unwrap()
                        ),
                    ))
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
