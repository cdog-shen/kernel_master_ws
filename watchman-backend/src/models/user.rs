use core::hash;

use bcrypt::hash;
use chrono;
use diesel::{prelude::*, Identifiable, Insertable, MysqlConnection, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::{
    config,
    models::schema::user::{self, dsl::*},
};

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize)]
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

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize)]
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

pub struct UserDataStream {}

pub struct AuthDataStream {}

impl UserModule {
    pub fn get_user_by_username(user_name: &str, conn: &mut MysqlConnection) -> Option<UserInfo> {
        match user
            .filter(is_enable.eq(1))
            .filter(username.eq(user_name))
            .select(UserInfo::as_select())
            .get_result::<UserInfo>(conn)
        {
            Ok(user_info) => Some(user_info),
            Err(_) => None,
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

    pub fn login(user_data: &BasicUserDataStream, conn: &mut MysqlConnection) -> Option<UserInfo> {
        match user
            .filter(is_enable.eq(1))
            .filter(username.eq(&user_data.username))
            .filter(passwd.eq(&user_data.passwd))
            .select(UserInfo::as_select())
            .get_result::<UserInfo>(conn)
        {
            Ok(user_identified) => Some(user_identified),
            Err(_) => None,
        }
    }
}
