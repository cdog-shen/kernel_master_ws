use core::num;

use actix_web::body::None;
use chrono::Utc;
use diesel::{prelude::*, Insertable, MysqlConnection, Queryable, Selectable};
use jsonwebtoken::{encode, EncodingKey, Header};
use log::debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use share_lib::cfg_reader::SECRET_KEY;

use crate::models::schema::token::{self, dsl::*};

static EXP_CONST: i64 = 60 * 60 * 24 * 7; // in seconds

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = token)]
pub struct TokenModel {
    pub tokenid: String,
    pub username: String,
    pub exp_time: chrono::NaiveDateTime,
}

impl<'a> TokenModel {
    pub fn find_token_by_username(
        user_token: &UserToken,
        conn: &mut MysqlConnection,
    ) -> Result<TokenModel, String> {
        match token
            .filter(username.eq(&user_token.user))
            .get_result::<TokenModel>(conn)
        {
            Ok(token_line) => Ok(token_line),
            Err(_) => Err(format!("can NOT find {}'s token.", &user_token.user)),
        }
    }

    pub fn update_token(
        user_token: &UserToken,
        conn: &mut MysqlConnection,
    ) -> Result<String, String> {
        match Self::find_token_by_username(&user_token, conn) {
            Ok(token_line) => {
                match diesel::update(token.find(token_line.username))
                    .set((
                        tokenid.eq(user_token.user.to_string()),
                        exp_time.eq(chrono::NaiveDateTime::from_timestamp(
                            user_token.exp.clone(),
                            0,
                        )),
                    ))
                    .execute(conn)
                {
                    Ok(num_of_change) => Ok(format!(
                        "{}'s token update. lines: {}",
                        &user_token.user, num_of_change
                    )),
                    Err(err) => Err(err.to_string()),
                }
            }
            Err(msg) => Err(msg),
        }
    }

    pub fn insert_new_token(
        user_token: &UserToken,
        conn: &mut MysqlConnection,
    ) -> Result<String, String> {
        match diesel::insert_into(token)
            .values(TokenModel {
                tokenid: user_token.uuid.clone(),
                username: user_token.user.clone(),
                exp_time: chrono::NaiveDateTime::from_timestamp(user_token.exp.clone(), 0),
            })
            .execute(conn)
        {
            Ok(num_of_change) => Ok(format!(
                "{}'s token updated. line: {}",
                &user_token.user, num_of_change
            )),
            Err(err) => Err(err.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserToken {
    pub user: String,
    pub uuid: String,
    pub exp: i64, // expiration
}

impl UserToken {
    pub fn encode_token(login_user: &String) -> String {
        debug!("Token Max Age: {}", EXP_CONST);

        let now = Utc::now();
        let exp = now.timestamp() + EXP_CONST; // Convert to Unix timestamp
        let payload = UserToken {
            exp,
            user: login_user.clone(),
            uuid: Uuid::new_v4().to_string(),
        };

        encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(&*SECRET_KEY.as_bytes()),
        )
        .unwrap()
    }

    // pub fn decode_token(token: &str) -> Result<UserToken, jsonwebtoken::errors::Error> {
    //     let decoding_key = EncodingKey::from_secret(&*SECRET_KEY.as_bytes());
    //     let decoded: UserToken = decode::<UserToken>(
    //         token,
    //         &decoding_key,
    //         &Validation::new(jsonwebtoken::Algorithm::HS256),
    //     )?;
    //     Ok(decoded)
    // }
}
