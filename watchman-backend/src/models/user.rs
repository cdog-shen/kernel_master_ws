use chrono;
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

// pub struct UserDataStream {}

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

    pub fn update_last_login(
        user_data: &BasicUserDataStream,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match Self::get_user_by_username(&user_data.username, conn) {
            Ok(user_line) => {
                if diesel::update(user.find(user_line.id))
                    .set(last_login.eq(chrono::Local::now().naive_utc()))
                    .execute(conn)
                    .is_err()
                {
                    Err((
                        UNKNOW_ERROR_CODE,
                        format!("{}'s last_login update filed.", &user_line.username),
                    ))
                } else {
                    Ok(format!("{}'s token update DONE.", &user_line.username))
                }
            }
            Err(msg) => Err(msg),
        }
    }
}
