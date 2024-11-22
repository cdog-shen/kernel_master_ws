use jsonwebtoken::{EncodingKey, Header};
use log::debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use share_lib::log_debug;

use crate::models;

static EXP_CONST: i64 = 60 * 60 * 24 * 7; // in seconds

#[derive(Serialize, Deserialize)]
pub struct UserToken {
    pub user: String,
    pub uuid: String,
    pub exp: i64, // expiration
}

// impl UserToken {
//     pub fn generate_token(login: &models::user::UserModule) -> String {
//         log_debug!("Token Max Age: {}", EXP_CONST);

//         let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nanosecond -> second
//         let payload = UserToken {
//             exp: now + EXP_CONST,
//             user: login.user.clone(),
//             uuid: Uuid::new_v4().to_string(),
//         };

//         jsonwebtoken::encode(
//             &Header::default(),
//             &payload,
//             &EncodingKey::from_secret(&KEY),
//         )
//         .unwrap()
//     }
// }
