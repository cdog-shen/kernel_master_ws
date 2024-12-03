use actix_web::{http::header::HeaderValue, web};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::models::{
    user::{BasicUserDataStream, UserModule},
    user_token::UserToken,
};

#[derive(Serialize, Deserialize)]
pub struct TokenBodyResponse {
    pub token: String,
    pub token_type: String,
}

pub fn login<'a>(
    user: BasicUserDataStream,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, TokenBodyResponse>, MailManErr> {
    let query_result = match UserModule::login(&user, &mut pool.get().unwrap()) {
        Some(user_info) => Some(user_info),
        None => None,
    };

    match query_result {
        Some(user_info) => {
            match UserToken::encode_token(user_info.username) {
                Ok(token_string) => {
                    let response = json!({
                        "token": token_string,
                        "token_type": "bearer",
                    });
                    serde_json::from_value(response)
                        .map_err(|e| {
                            MailManErr::new(500, "Internal Server Error", e.to_string(), 1)
                        })
                        .map(|token_response| MailManOk {
                            code: 200,
                            key: "JWT generate DONE",
                            data: token_response,
                        })
                }
                Err(_) => Err(MailManErr::new(
                    500,
                    "Internal Server Error",
                    "Can NOT generate JWT token",
                    1,
                )),
            }
        }
        None => Err(MailManErr::new(400, "Bad Request", user.username, 1)),
    }
}
