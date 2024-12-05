use chrono;
use diesel::{prelude::*, Insertable, MysqlConnection, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::models::schema::user::{self, dsl::*};

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

    pub fn login(
        user_data: &BasicUserDataStream,
        conn: &mut MysqlConnection,
    ) -> Result<UserInfo, String> {
        match user
            .filter(is_enable.eq(1))
            .filter(username.eq(&user_data.username))
            .filter(passwd.eq(&user_data.passwd))
            .select(UserInfo::as_select())
            .get_result::<UserInfo>(conn)
        {
            Ok(user_identified) => Ok(user_identified),
            Err(err) => Err(format!("Login filed: {} - {}", &user_data.username, err.to_string())),
        }
    }

    pub fn update_last_login(
        user_data: &BasicUserDataStream,
        conn: &mut MysqlConnection,
    ) -> Result<String, String> {
        match Self::get_user_by_username(&user_data.username, conn) {
            Some(user_line) => {
                if diesel::update(user.find(user_line.id))
                    .set(last_login.eq(chrono::Utc::now().naive_utc()))
                    .execute(conn)
                    .is_err()
                {
                    Err(format!("{}'s last_login update filed.", &user_line.username))
                } else {
                    Ok(format!("{}'s token update DONE.", &user_line.username))
                }
            }
            None => Err(format!("can NOT find user {}", &user_data.username)),
        }
    }
}
