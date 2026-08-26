use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::schema::notification_aliases::{self, dsl};

static UNKNOWN_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = notification_aliases, check_for_backend(diesel::pg::Pg))]
pub struct NotificationAlias {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub recipients: Value,
    pub is_enabled: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = notification_aliases)]
pub struct NewNotificationAlias {
    pub name: String,
    pub description: Option<String>,
    pub recipients: Value,
    pub is_enabled: Option<bool>,
}

#[derive(AsChangeset, Debug, Serialize, Deserialize)]
#[diesel(table_name = notification_aliases)]
pub struct UpdateNotificationAlias {
    pub name: Option<String>,
    pub description: Option<String>,
    pub recipients: Option<Value>,
    pub is_enabled: Option<bool>,
    pub updated_at: Option<NaiveDateTime>,
}

impl NotificationAlias {
    /// 根据名称获取 alias
    pub fn get_by_name(
        alias_name: &str,
        conn: &mut PgConnection,
    ) -> Result<Option<Self>, (u8, String)> {
        match dsl::notification_aliases
            .filter(dsl::name.eq(alias_name))
            .filter(dsl::is_enabled.eq(Some(true)))
            .select(NotificationAlias::as_select())
            .first(conn)
        {
            Ok(alias) => Ok(Some(alias)),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 获取所有 alias
    pub fn get_all(conn: &mut PgConnection) -> Result<Vec<Value>, (u8, String)> {
        match dsl::notification_aliases
            .select(NotificationAlias::as_select())
            .load(conn)
        {
            Ok(aliases) => Ok(aliases
                .into_iter()
                .map(|a| serde_json::to_value(&a).unwrap())
                .collect()),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 创建新 alias
    pub fn create(
        new_alias: &NewNotificationAlias,
        conn: &mut PgConnection,
    ) -> Result<i32, (u8, String)> {
        match diesel::insert_into(dsl::notification_aliases)
            .values(new_alias)
            .returning(dsl::id)
            .get_result::<i32>(conn)
        {
            Ok(new_id) => Ok(new_id),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 更新 alias
    pub fn update(
        alias_id: i32,
        update: &UpdateNotificationAlias,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(dsl::notification_aliases.filter(dsl::id.eq(alias_id)))
            .set(update)
            .execute(conn)
        {
            Ok(num) => match num {
                0 => Err((
                    BAD_REQUEST_CODE,
                    format!("Alias id: {} not found", alias_id),
                )),
                _ => Ok(num),
            },
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 删除 alias
    pub fn delete(alias_id: i32, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(dsl::notification_aliases.filter(dsl::id.eq(alias_id))).execute(conn) {
            Ok(num) => match num {
                0 => Err((
                    BAD_REQUEST_CODE,
                    format!("Alias id: {} not found", alias_id),
                )),
                _ => Ok(num),
            },
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }
}
