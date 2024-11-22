use diesel::Queryable;
use serde::{Deserialize, Serialize};
use time;

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
