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

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = cloud_account)]
pub struct CloudAccountInfo {
    pub id: Option<u32>,
    pub cloud_provider: Option<String>,
    pub nick_name: Option<String>,
    pub ak: Option<String>,
    pub sk: Option<String>,
    pub is_enable: Option<u8>,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub comment: Option<String>,
}

impl CloudAccountInfo {
    fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(CloudAccountInfo {
            id: map.get("id").map(|value| value.as_u64().unwrap() as u32),
            cloud_provider: map
                .get("cloud_provider")
                .map(|value| value.as_str().unwrap().to_string()),
            nick_name: map
                .get("nick_name")
                .map(|value| value.as_str().unwrap().to_string()),
            ak: map
                .get("ak")
                .map(|value| value.as_str().unwrap().to_string()),
            sk: map
                .get("sk")
                .map(|value| value.as_str().unwrap().to_string()),
            is_enable: map
                .get("is_enable")
                .map(|value| value.as_u64().unwrap() as u8),
            update_time: Some(Local::now().naive_local()),
            comment: map
                .get("comment")
                .map(|value| value.as_str().unwrap().to_string()),
        })
    }
}

impl CloudAccountModel {
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
                _ => (UNKNOW_ERROR_CODE, format!("get account error: {e}")),
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
                    let item_map = serde_json::to_value(&item)
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .clone();
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
        let new_account = match CloudAccountInfo::from_map(account_map) {
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
        let update_account = match CloudAccountInfo::from_map(account_map) {
            Ok(account) => {
                if let Some(_uid) = account.id {
                    account
                } else {
                    return Err((UNKNOW_ERROR_CODE, "id is required".to_string()));
                }
            }
            Err(e) => return Err((UNKNOW_ERROR_CODE, e)),
        };
        match diesel::update(cloud_account.filter(id.eq(update_account.id.unwrap())))
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
