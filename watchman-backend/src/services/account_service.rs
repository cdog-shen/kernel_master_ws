use actix_web::web;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::models::{
    user::{BasicUserDataStream, UserModel, UserUpdate},
    user_token::{TokenModel, UserToken},
};

/// token Response json data
#[derive(Serialize, Deserialize)]
pub struct TokenBodyResponse {
    pub token: String,
    pub token_type: String,
}

/// login api logic
///
/// 1. check user name and password
/// 2. generate token and json
/// 3. update user's last login time
/// 4. save token to DB
pub fn login<'a>(
    user: BasicUserDataStream,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, TokenBodyResponse>, MailManErr> {
    // 1. check user name and password
    let query_result = match UserModel::login(&user, &mut pool.get().unwrap()) {
        Ok(user_info) => user_info,
        Err(msg) => {
            return Err(MailManErr::new(400, "Bad Request", msg.1, 1));
        }
    };

    // 2. generate token and json
    let token_obj = UserToken::new(&query_result.username);

    let response = match token_obj.encode_token() {
        Ok(jwt) => json!({
            "token": jwt,
            "token_type": "bearer",
        }),
        Err(err) => return Err(MailManErr::new(500, "Internal Server Error", err.1, 1)),
    };

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

    // 3. update user's last login time
    match UserModel::update_last_login(&user, &mut pool.get().unwrap()) {
        Ok(msg) => {
            MailManOk::<String>::new(
                200,
                format!("update_last_login call success - {}", msg).as_str(),
                Some(msg),
            );
        }
        Err(msg) => return Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
    }

    // 4. save token to DB
    match TokenModel::update_token(&token_obj, &mut pool.get().unwrap()) {
        Ok(msg) => {
            MailManOk::<String>::new(
                200,
                format!("update_token call success - {}", msg).as_str(),
                Some(msg),
            );
        }
        Err(msg) => {
            if msg.0 == 1 {
                match TokenModel::new_token(&token_obj, &mut pool.get().unwrap()) {
                    Ok(msg) => {
                        MailManOk::<String>::new(
                            200,
                            format!("insert_new_token call success - {}", msg).as_str(),
                            Some(msg),
                        );
                    }
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

/// signup api logic
pub fn new_user<'a>(
    user_to_creat: BasicUserDataStream,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr> {
    match UserModel::new_user(&user_to_creat, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "New user creat success", Some(msg))),
        Err(msg) => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
    }
}

/// logout api logic
pub fn logout<'a>(
    username_to_logout: String,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr> {
    match TokenModel::delete(&username_to_logout, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "logout success", Some(msg))),
        Err(msg) => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
    }
}

/// user_update api logic
pub fn user_update<'a>(
    user_info: UserUpdate,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr> {
    match UserModel::update_user_by_id(&user_info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "info updated", Some(msg))),
        Err(msg) => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
    }
}
