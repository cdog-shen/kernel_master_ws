use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, EncodingKey, Header, Validation};
use log::debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use share_lib::cfg_reader::SECRET_KEY;

use crate::models::user::BasicUserDataStream;

static EXP_CONST: i64 = 60 * 60 * 24 * 7; // in seconds

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
