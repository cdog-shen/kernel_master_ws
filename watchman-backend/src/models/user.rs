use diesel::Queryable;
use serde::{Deserialize, Serialize};
use time;

#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct UserModule {
    pub id: i32,
    pub password: String,
    pub username: String,
    pub group: String,
    pub is_superuser: bool,
    pub email: String,
    pub is_staff: bool,
    pub is_active: bool,
    pub name: String,
    pub phone: Option<String>,
    pub date_joined: time::PrimitiveDateTime,
    pub last_login: Option<time::PrimitiveDateTime>,
}
