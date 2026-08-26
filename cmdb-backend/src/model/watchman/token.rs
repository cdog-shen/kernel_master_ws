use chrono::{self};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::model::schema::token_table::{self, dsl::*};

static UNKNOW_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = token_table)]
pub struct TokenModel {
    pub tokenid: String,
    pub username: String,
    pub exp_time: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = token_table)]
pub struct TokenInfo {
    pub tokenid: Option<String>,
    pub username: Option<String>,
    pub exp_time: Option<chrono::NaiveDateTime>,
}

impl TokenInfo {
    pub fn from_map(map: Map<String, Value>) -> Result<Self, String> {
        Ok(TokenInfo {
            tokenid: match map.get("tokenid") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            username: match map.get("username") {
                Some(value) => value.as_str().map(|s| s.to_string()),
                None => None,
            },
            exp_time: match map.get("exp_time") {
                Some(value) => value.as_str().and_then(|s| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()
                }),
                None => None,
            },
        })
    }
}

impl TokenModel {
    pub fn get_model_info_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = token_table.into_boxed().select(TokenModel::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "username" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(username.eq(value));
                    }
                }
                _ => continue,
            }
        }

        match query.get_results::<TokenModel>(conn) {
            Ok(vec_item_info) => {
                let vec_value: Vec<Value> = vec_item_info
                    .into_iter()
                    .map(|item| serde_json::to_value(item).unwrap_or(Value::Null))
                    .collect();
                Ok(vec_value)
            }
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}

impl TokenModel {
    pub fn new(info: &TokenInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::insert_into(token_table).values(info).execute(conn) {
            Ok(num_of_eff) => Ok(num_of_eff),
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn update(info: &TokenInfo, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::update(token_table.filter(tokenid.eq(info.tokenid.as_ref().unwrap())))
            .set(info)
            .execute(conn)
        {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((
                    BAD_REQUEST_CODE,
                    format!("tokenid: {} not found", info.tokenid.as_ref().unwrap()),
                )),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }

    pub fn delete(_tokenid: &str, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(token_table.filter(tokenid.eq(_tokenid))).execute(conn) {
            Ok(num_of_eff) => match num_of_eff {
                0 => Err((BAD_REQUEST_CODE, format!("tokenid: {} not found", _tokenid))),
                _ => Ok(num_of_eff),
            },
            Err(e) => Err((UNKNOW_ERROR_CODE, e.to_string())),
        }
    }
}
