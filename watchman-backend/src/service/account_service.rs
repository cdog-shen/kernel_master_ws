use std::collections::HashMap;

use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::{access::*, group::*, service::*, user::*, user_token::*};

/// token Response json data
#[derive(Serialize, Deserialize)]
pub struct TokenBodyResponse {
    pub uid: u32,
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
    user: UserInputStream,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, TokenBodyResponse>, MailManErr<'a, String>> {
    // 1. check user name and password
    let query_result = match UserModel::login(&user, &mut pool.get().unwrap()) {
        Ok(user_info) => user_info,
        Err(msg) => {
            return Err(MailManErr::new(400, "Bad Request", Some(msg.1), 1));
        }
    };

    // 2. generate token and json
    let token_obj = UserToken::new(&query_result.username.unwrap());

    let response = match token_obj.encode_token() {
        Ok(jwt) => serde_json::json!({
            "uid": query_result.id,
            "token": jwt,
            "token_type": "bearer",
        }),
        Err(err) => {
            return Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(err.1),
                1,
            ));
        }
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
                Some(err.to_string()),
                1,
            ));
        }
    };

    // 3. update user's last login time
    match UserModel::update_last_login(&user, &mut pool.get().unwrap()) {
        Ok(msg) => {
            MailManOk::<String>::new(
                200,
                format!("update_last_login call success - {msg}").as_str(),
                Some(msg),
            );
        }
        Err(msg) => {
            return Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(msg.1),
                1,
            ));
        }
    }

    // 4. save token to DB
    match TokenModel::update_token(&token_obj, &mut pool.get().unwrap()) {
        Ok(msg) => {
            MailManOk::<String>::new(
                200,
                format!("update_token call success - {msg}").as_str(),
                Some(msg),
            );
        }
        Err(msg) => {
            if msg.0 == 1 {
                match TokenModel::new_token(&token_obj, &mut pool.get().unwrap()) {
                    Ok(msg) => {
                        MailManOk::<String>::new(
                            200,
                            format!("insert_new_token call success - {msg}").as_str(),
                            Some(msg),
                        );
                    }
                    Err(msg) => {
                        return Err(MailManErr::new(
                            500,
                            "Internal Server Error",
                            Some(msg.1),
                            1,
                        ));
                    }
                }
            } else {
                return Err(MailManErr::new(
                    500,
                    "Internal Server Error",
                    Some(msg.1),
                    1,
                ));
            }
        }
    }

    output
}

/// signup api logic
pub fn new_user<'a>(
    user_to_creat: UserInputStream,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<String>> {
    match UserModel::new_user(&user_to_creat, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "New user creat success", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    }
}

/// logout api logic
pub fn logout<'a>(
    user_info: UserInputStream,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<String>> {
    match TokenModel::delete(&user_info.username.unwrap(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Logout success", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    }
}

/// user_update api logic
pub fn user_update<'a>(
    user_info: UserInputStream,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<String>> {
    match UserModel::update_user_by_id(&user_info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "User info updated", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    }
}

/// get all user info
pub fn get_all<'a>(
    filter: Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Vec<UserOutputStream>>, MailManErr<'a, String>> {
    match UserModel::get_user_info_with_filter(&filter, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "All user info", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(msg.1),
                1,
            )),
            _ => Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    }
}

/// get user's all info
pub fn get_me(
    id: i32,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HashMap<String, serde_json::Value>, MailManErr<'_, String>> {
    let mut result: HashMap<String, serde_json::Value> = HashMap::new();

    let user_basic_info = match UserModel::get_user_by_id(id, &mut pool.get().unwrap()) {
        Ok(ubi) => ubi,
        Err(msg) => match msg.0 {
            0 => {
                return Err(MailManErr::new(
                    500,
                    "Internal Server Error",
                    Some(msg.1),
                    1,
                ));
            }
            _ => return Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    };

    let user_group_info = match GroupModel::get_groups_by_uid(id, &mut pool.get().unwrap()) {
        Ok(ugi) => ugi,
        Err(msg) => match msg.0 {
            0 => {
                return Err(MailManErr::new(
                    500,
                    "Internal Server Error",
                    Some(msg.1),
                    1,
                ));
            }
            _ => return Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    };

    let user_access_info = match AccessModel::get_access_by_gids(
        &user_group_info.iter().map(|group| group.id).collect(),
        &mut pool.get().unwrap(),
    ) {
        Ok(uai) => uai,
        Err(msg) => match msg.0 {
            0 => {
                return Err(MailManErr::new(
                    500,
                    "Internal Server Error",
                    Some(msg.1),
                    1,
                ));
            }
            _ => return Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    };

    let user_service_info = match ServiceModel::get_services_by_id(
        &user_access_info
            .iter()
            .map(|access| access.service_id)
            .collect(),
        &mut pool.get().unwrap(),
    ) {
        Ok(usi) => usi,
        Err(msg) => match msg.0 {
            0 => {
                return Err(MailManErr::new(
                    500,
                    "Internal Server Error",
                    Some(msg.1),
                    1,
                ));
            }
            _ => return Err(MailManErr::new(400, "Bad requests", Some(msg.1), 1)),
        },
    };

    result.insert("ubi".to_string(), serde_json::json!(&user_basic_info));
    result.insert("ugi".to_string(), serde_json::json!(&user_group_info));
    result.insert("uai".to_string(), serde_json::json!(&user_access_info));
    result.insert("usi".to_string(), serde_json::json!(&user_service_info));

    Ok(result)
}
