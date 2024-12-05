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
    user_token::{TokenModel, UserToken},
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
        Ok(user_info) => user_info,
        Err(msg) => {
            return Err(MailManErr::new(400, "Bad Request", msg.1, 1));
        }
    };

    let token_obj = UserToken::new(&query_result.username);

    let response = json!({
        "token": token_obj.encode_token(),
        "token_type": "bearer",
    });

    let output = match serde_json::from_value(response) {
        Ok(token_response_warp) => Ok(MailManOk::<TokenBodyResponse>::new(
            200,
            "Login in success.",
            token_response_warp,
        )),
        Err(err) => {
            return Err(MailManErr::new(
                500,
                "Internal Server Error",
                err.to_string(),
                1,
            ))
        }
    };

    match UserModule::update_last_login(&user, &mut pool.get().unwrap()) {
        Ok(_) => (),
        Err(msg) => return Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
    }

    match TokenModel::update_token(&token_obj, &mut pool.get().unwrap()) {
        Ok(_) => (),
        Err(msg) => {
            if msg.0 == 1 {
                match TokenModel::insert_new_token(&token_obj, &mut pool.get().unwrap()) {
                    Ok(_) => (),
                    Err(msg) => {
                        return Err(MailManErr::new(500, "Internal Server Error", msg.1, 1))
                    }
                }
            } else {
                return Err(MailManErr::new(500, "Internal Server Error", msg.1, 1));
            }
        }
    }

    return output;
}
