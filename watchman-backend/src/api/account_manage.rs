use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};

use crate::utils::err_mapping::MailManErrResponser;

use crate::{
    models::user::BasicUserDataStream, services::account_service,
};

// POST api/auth/login
pub async fn login(
    login_json: web::Json<BasicUserDataStream>,
    pool: web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match account_service::login(login_json.0, &pool) {
        Ok(token_res) => Ok(HttpResponse::Ok().json(token_res)),
        Err(err_mm) => Err(MailManErrResponser { mme_msg: err_mm.msg }),
    }
}
