use jsonwebtoken::{EncodingKey, Header};
use log::debug;
use serde::{Deserialize, Serialize};

static EXP_CONST: i64 = 60 * 60 * 24 * 7; // in seconds

#[derive(Serialize, Deserialize)]
pub struct UserToken {
    // issued at
    pub iat: i64,
    // expiration
    pub exp: i64,
    // data
    pub user: String,
    pub login_session: String,
}