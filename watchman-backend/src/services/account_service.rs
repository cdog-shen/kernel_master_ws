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
            return Err(MailManErr {
                code: 400,
                key: "Bad Request",
                msg: msg.1,
                level: 1,
            });
        }
    };

    let token_obj = UserToken::new(&query_result.username);

    let response = json!({
        "token": token_obj.encode_token(),
        "token_type": "bearer",
    });

    let output = match serde_json::from_value(response) {
        Ok(token_response_warp) => Ok(MailManOk::<TokenBodyResponse> {
            code: 200,
            key: "JWT generate DONE",
            data: token_response_warp,
        }),
        Err(err) => {
            return Err(MailManErr {
                code: 500,
                key: "Internal Server Error",
                msg: err.to_string(),
                level: 1,
            })
        }
    };

    match UserModule::update_last_login(&user, &mut pool.get().unwrap()) {
        Ok(_) => (),
        Err(msg) => {
            return Err(MailManErr {
                code: 500,
                key: "Internal Server Error",
                msg: msg.1,
                level: 1,
            })
        }
    }

    match TokenModel::update_token(&token_obj, &mut pool.get().unwrap()) {
        Ok(_) => (),
        Err(msg) => {
            if msg.0 == 1 {
                match TokenModel::insert_new_token(&token_obj, &mut pool.get().unwrap()) {
                    Ok(_) => (),
                    Err(msg) => {
                        return Err(MailManErr {
                            code: 500,
                            key: "Internal Server Error",
                            msg: msg.1,
                            level: 1,
                        })
                    }
                }
            } else {
                return Err(MailManErr {
                    code: 500,
                    key: "Internal Server Error",
                    msg: msg.1,
                    level: 1,
                });
            }
        }
    }

    return output;
}
