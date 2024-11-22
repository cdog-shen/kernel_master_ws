use core::hash;

use bcrypt::hash;
use diesel::{prelude::*, Identifiable, Insertable, MysqlConnection, Queryablem};
use serde::{Deserialize, Serialize};
use time;

use crate::config;

#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct UserModule {
    pub id: i32,
    pub user: String,
    pub passwd: String,
    pub is_enable: bool,
    pub name: String,
    pub contact: String,
    pub group: String,
    pub date_joined: time::PrimitiveDateTime,
    pub last_login: Option<time::PrimitiveDateTime>,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = users)]
pub struct SignUpDataStream {
    pub user: String,
    pub passwd: String,
}

pub struct UserDataStream {}

pub struct AuthDataStream {}

impl UserModule {
    pub fn signup(
        new_user: SignUpDataStream,
        conn: &mut MysqlConnection,
    ) -> Result<String, String> {
        if Self::find_user_by_username(&new_user.user, conn).is_err() {
            let new_user = SignUpDataStream {
                passwd: new_user.passwd,
                ..new_user
            };
            diesel::insert_into(users).values(new_user).execute(conn);
            Ok(constants::MESSAGE_SIGNUP_SUCCESS.to_string())
        } else {
            Err(format!(
                "User '{}' is already registered",
                &new_user.username
            ))
        }
    }
}
