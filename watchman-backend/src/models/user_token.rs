use chrono::Local;
use diesel::{
    prelude::*, result::Error::NotFound, Insertable, MysqlConnection, Queryable, Selectable,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use log::debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use share_lib::{cfg_reader::SECRET_KEY, log_debug};

use crate::models::schema::token_table::{self, dsl::*};

// expire time const var
static EXP_CONST: i64 = 60 * 60 * 24 * 7; // in seconds Week
                                          // Error status const code
static NOT_FOUND_CODE: u8 = 1;
static UNKNOW_ERROR_CODE: u8 = 0;

/// The structure of the token stored in the database.
///
/// - tokenid
///
///     Token's unique identifier (UUIDv4 String)
///
/// - username
///
///     User Associated with the Token (String)
///
/// - exp_time
///
///     Token's expire time (%Y-%m-%d %H:%M:%S)
#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = token_table)]
pub struct TokenModel {
    pub tokenid: String,
    pub username: String,
    pub exp_time: chrono::NaiveDateTime,
}

// query implement
impl<'a> TokenModel {
    /// find token by username
    pub fn find_token_by_username(
        user_token: &UserToken,
        conn: &mut MysqlConnection,
    ) -> Result<TokenModel, (u8, String)> {
        match token_table
            .filter(username.eq(&user_token.user))
            .get_result::<TokenModel>(conn)
        {
            Ok(token_line) => Ok(token_line),
            Err(NotFound) => Err((
                NOT_FOUND_CODE,
                format!("can NOT find {}'s token.", &user_token.user),
            )),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow error {}", e.to_string()))),
        }
    }

    /// check token validation
    pub fn token_ckeck(
        decode_token: &TokenModel,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match token_table
            .filter(username.eq(&decode_token.username))
            .filter(tokenid.eq(&decode_token.tokenid))
            .get_result::<TokenModel>(conn)
        {
            Ok(token_line) => {
                if Local::now().naive_local() < token_line.exp_time {
                    Ok("token valid".to_string())
                } else {
                    Err((NOT_FOUND_CODE, format!("token expired.")))
                }
            }
            Err(NotFound) => Err((NOT_FOUND_CODE, format!("token invaild."))),
            Err(e) => Err((UNKNOW_ERROR_CODE, format!("Unknow error {}", e.to_string()))),
        }
    }
}

// update implement
impl TokenModel {
    /// crate a token data in DB
    #[allow(deprecated)]
    pub fn new_token(
        user_token: &UserToken,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        match diesel::insert_into(token_table)
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
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    /// update a token in DB
    #[allow(deprecated)]
    pub fn update_token(
        user_token: &UserToken,
        conn: &mut MysqlConnection,
    ) -> Result<String, (u8, String)> {
        let target = token_table.filter(username.eq(&user_token.user));

        let _: Result<i64, (u8, String)> = match target.count().get_result(conn) {
            Ok(c) => match c {
                1 => Ok(1),
                _ => {
                    return Err((
                        NOT_FOUND_CODE,
                        format!("can NOT find specific token for {}.", &user_token.user),
                    ))
                }
            },
            Err(e) => return Err((UNKNOW_ERROR_CODE, e.to_string())),
        };

        match diesel::update(target)
            .set((
                tokenid.eq(user_token.uuid.to_string()),
                exp_time.eq(chrono::NaiveDateTime::from_timestamp(
                    user_token.exp.clone(),
                    0,
                )),
            ))
            .execute(conn)
        {
            Ok(num_of_eff) => Ok(format!(
                "{}'s token update. lines: {}",
                &user_token.user, num_of_eff
            )),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    /// remove token
    pub fn delete(user_name: &String, conn: &mut MysqlConnection) -> Result<String, (u8, String)> {
        let target = token_table.filter(username.eq(user_name));

        match diesel::delete(target).execute(conn) {
            Ok(num_of_eff) => Ok(format!(
                "{}'s token update. lines: {}",
                user_name, num_of_eff
            )),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }
}

/// Token data struct in service. which can map to TokenModel
///
/// - user (UUIDv4 String) -> TokenModel.username
/// - uuid (String) -> TokenModel.tokenid
/// - exp (i64 timestamp) -> TokenModel.exp_time
#[derive(Serialize, Deserialize, Debug)]
pub struct UserToken {
    pub user: String,
    pub uuid: String,
    pub exp: i64, // expiration
}

impl UserToken {
    /// generate a token claim object
    #[allow(deprecated)]
    pub fn new(login_user: &String) -> UserToken {
        UserToken {
            user: login_user.clone(),
            uuid: Uuid::new_v4().to_string(),
            exp: Local::now().naive_local().timestamp() + EXP_CONST,
        }
    }

    /// map a UserToken object to a TokenModel object
    #[allow(deprecated)]
    pub fn map_to_tm(self) -> TokenModel {
        TokenModel {
            tokenid: self.uuid,
            username: self.user,
            exp_time: chrono::NaiveDateTime::from_timestamp(self.exp, 0),
        }
    }

    /// encode a token claim object as JWT string
    pub fn encode_token(&self) -> Result<String, (u8, String)> {
        debug!("Token Max Age: {}", EXP_CONST);

        let payload = self;

        match encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(&*SECRET_KEY.as_bytes()),
        ) {
            Ok(jwt) => Ok(jwt),
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }

    /// decode a JWT string back to token claim object and map it into a TokenModel
    pub fn decode_token(token_str: String) -> Result<TokenModel, (u8, String)> {
        let decoding_key = DecodingKey::from_secret(&*SECRET_KEY.as_bytes());
        match decode::<UserToken>(
            &token_str,
            &decoding_key,
            &Validation::new(jsonwebtoken::Algorithm::default()),
        ) {
            Ok(token_obj) => {
                log_debug!("{:?}", token_obj);
                Ok(token_obj.claims.map_to_tm())
            }
            Err(err) => Err((UNKNOW_ERROR_CODE, err.to_string())),
        }
    }
}
