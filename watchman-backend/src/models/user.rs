use chrono::{self, Local};
use diesel::{
    prelude::*, result::Error::NotFound, Insertable, MysqlConnection, Queryable, Selectable,
};
use serde::{Deserialize, Serialize};

use crate::models::schema::user::{self, dsl::*};

static NOT_FOUND_CODE: u8 = 1;
static UNKNOW_ERROR_CODE: u8 = 0;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = user)]
pub struct UserModule {
    pub id: u32,
    pub username: String,
    pub passwd: String,
    pub is_enable: u8,
    pub name: Option<String>,
    pub contact: Option<String>,
    pub groups: Option<String>,
    pub date_joined: Option<chrono::NaiveDateTime>,
    pub last_login: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = user)]
pub struct UserInfo {
    pub id: u32,
    pub username: String,
    pub is_enable: u8,
    pub name: Option<String>,
    pub contact: Option<String>,
    pub groups: Option<String>,
    pub date_joined: Option<chrono::NaiveDateTime>,
    pub last_login: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = user)]
pub struct BasicUserDataStream {
    pub username: String,
    pub passwd: String,
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = user)]
pub struct UserUpdate {
    pub id: u32,
    pub username: Option<String>,
    pub passwd: Option<String>,
    pub is_enable: Option<u8>,
    pub name: Option<String>,
    pub contact: Option<String>,
    pub groups: Option<String>,
}

// pub struct AuthDataStream {}

impl UserModule {
    pub fn get_user_by_username(
        user_name: &str,
        conn: &mut MysqlConnection,
    ) -> Result<UserInfo, (u8, String)> {
        match user
            .filter(is_enable.eq(1))
            .filter(username.eq(user_name))
            .select(UserInfo::as_select())
            .get_result::<UserInfo>(conn)
        {
            Ok(user_info) => Ok(user_info),
            Err(NotFound) => Err((NOT_FOUND_CODE, format!("NotFound {}.", user_name))),
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e.to_string()),
            )),
        }
    }

    pub fn get_user_info(conn: &mut MysqlConnection) -> Vec<UserInfo> {
        match user
            .select(UserInfo::as_select()) // 选择 UserInfo 结构体中定义的字段
            .get_results::<UserInfo>(conn)
        {
            Ok(vec_user_info) => vec_user_info,
            Err(_) => vec![],
        }
    }

    pub fn login(
        user_data: &BasicUserDataStream,
        conn: &mut MysqlConnection,
    ) -> Result<UserInfo, (u8, String)> {
        match user
            .filter(is_enable.eq(1))
            .filter(username.eq(&user_data.username))
            .filter(passwd.eq(&user_data.passwd))
            .select(UserInfo::as_select())
            .get_result::<UserInfo>(conn)
        {
            Ok(user_identified) => Ok(user_identified),
            Err(NotFound) => Err((
                NOT_FOUND_CODE,
                format!("Login filed: {}.", &user_data.username),
            )),
            Err(e) => Err((
                UNKNOW_ERROR_CODE,
                format!("Unknow Error: {}.", e.to_string()),
            )),
        }
    }
}

impl UserModule {
    pub fn new_user(
        user_data: &BasicUserDataStream,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::insert_into(user)
            .values((
                username.eq(&user_data.username),
                passwd.eq(&user_data.passwd),
                date_joined.eq(Local::now().naive_local()),
            ))
            .execute(conn)
        {
            Ok(num_of_change) => Ok(format!(
                "User {} created. line: {}",
                &user_data.username, num_of_change
            )),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    pub fn update_user_info(
        user_update: &UserUpdate,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::update(user.find(user_update.id))
            .set(user_update)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(format!(
                "{}'s data updated. lines: {}",
                user_update.id, num_of_eff
            )),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update_last_login(
        user_data: &BasicUserDataStream,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        let target = user.filter(username.eq(&user_data.username));

        let _: Result<i64, (u8, String)> = match target.count().get_result(conn) {
            Ok(c) => match c {
                1 => Ok(1),
                _ => {
                    return Err((
                        NOT_FOUND_CODE,
                        format!("can NOT find specific user: {}.", &user_data.username),
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
                &user_data.username, num_of_change
            )),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }
}
