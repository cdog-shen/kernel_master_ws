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
        Some(user_info) => user_info,
        None => {
            return Err(MailManErr::new(
                400,
                "Bad Request",
                user.username,
                1,
            ));
        }
    };

    let response = json!({
        "token": UserToken::encode_token(&query_result.username),
        "token_type": "bearer",
    });

    match serde_json::from_value(response) {
        Ok(token_response) => Ok(MailManOk {
            code: 200,
            key: "JWT generate DONE",
            data: token_response,
        }),
        Err(err) => Err(MailManErr::new(
            500,
            "Internal Server Error",
            err.to_string(),
            1,
        )),
    }
}
