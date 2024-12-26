use chrono::{self, Local};
use diesel::{prelude::*, result::Error::NotFound};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::models::schema::cloud_account::{self, dsl::*};

static NOT_FOUND_CODE: u8 = 1;
static UNKNOW_ERROR_CODE: u8 = 0;
// static TMI_ERROR_CODE: u8 = 2;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = cloud_account)]
pub struct CloudAccountModel {
    pub id: u32,
    pub cloud_provider: String,
    pub nick_name: String,
    pub ak: String,
    pub sk: String,
    pub is_enable: u8,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub comment: Option<String>,
}

impl CloudAccountModel {
    fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(CloudAccountModel {
            id: map
                .get("id")
                .and_then(Value::as_u64)
                .and_then(|v| u32::try_from(v).ok())
                .ok_or("id missing")?,
            cloud_provider: map
                .get("cloud_provider")
                .and_then(Value::as_str)
                .ok_or("cloud_provider missing")?
                .to_string(),
            nick_name: map
                .get("nick_name")
                .and_then(Value::as_str)
                .ok_or("nick_name missing")?
                .to_string(),
            ak: map
                .get("ak")
                .and_then(Value::as_str)
                .ok_or("ak missing")?
                .to_string(),
            sk: map
                .get("sk")
                .and_then(Value::as_str)
                .ok_or("sk missing")?
                .to_string(),
            is_enable: map
                .get("is_enable")
                .and_then(Value::as_u64)
                .and_then(|v| u8::try_from(v).ok())
                .ok_or("is_enable missing")?,
            update_time: map
                .get("update_time")
                .and_then(Value::as_str)
                .map(|s| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                        .map_err(|_| "update_time parse error")
                })
                .transpose()?,
            comment: map
                .get("comment")
                .and_then(Value::as_str)
                .map(|s| s.to_string()),
        })
    }

    /// get account by id
    pub fn get_account_by_id(
        account_id: u32,
        conn: &mut MysqlConnection,
    ) -> Result<Self, (u8, String)> {
        cloud_account
            .filter(id.eq(account_id))
            .first::<CloudAccountModel>(conn)
            .map_err(|e| match e {
                NotFound => (NOT_FOUND_CODE, "account not found".to_string()),
                _ => (UNKNOW_ERROR_CODE, format!("get account error: {}", e)),
            })
    }

    /// get account full data
    pub fn get_user_info_with_filter(
        filter: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = cloud_account
            .into_boxed()
            .select(CloudAccountModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "cloud_provider" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(cloud_provider.eq(value));
                    }
                }
                "nick_name" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(nick_name.eq(value));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<CloudAccountModel>(conn) {
            Ok(vec_item_info) => Ok(vec_item_info
                .into_iter()
                .map(|item| {
                    let mut item_map = serde_json::to_value(&item)
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .clone();
                    item_map.insert(
                        "create_at".to_string(),
                        Value::String(Local::now().naive_local().to_string()),
                    );
                    item_map.insert(
                        "update_at".to_string(),
                        Value::String(Local::now().naive_local().to_string()),
                    );
                    Value::Object(item_map)
                })
                .collect()),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}

impl CloudAccountModel {
    pub fn new_account(
        account_map: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<usize, (u8, String)> {
        let new_account = match CloudAccountModel::from_map(account_map) {
            Ok(account) => account,
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };
        match diesel::insert_into(cloud_account)
            .values(&new_account)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update_account(
        account_map: Map<String, Value>,
        conn: &mut MysqlConnection,
    ) -> Result<usize, (u8, String)> {
        let update_account = match CloudAccountModel::from_map(account_map) {
            Ok(account) => account,
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };
        match diesel::update(cloud_account.filter(id.eq(update_account.id)))
            .set(&update_account)
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete_account(
        account_id: u32,
        conn: &mut MysqlConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::delete(cloud_account.filter(id.eq(account_id))).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
