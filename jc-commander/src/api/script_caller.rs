use std::sync::Arc;

use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::Value;

use share_lib::err_mapping::MailManErrResponser;

use crate::service::script_caller;

// send sync task to message queue
//
// handler 只做请求清洗（取出 JSON body），编排全部在 service 层
pub async fn call_sync(
    req: web::Json<Value>,
    db_pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
    mq_pool: web::Data<Arc<lapin::Connection>>,
) -> Result<HttpResponse, MailManErrResponser> {
    script_caller::call_sync(req.into_inner(), &db_pool, &mq_pool).await
}

// send async task to message queue
pub async fn call_async(
    req: web::Json<Value>,
    db_pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
    mq_pool: web::Data<Arc<lapin::Connection>>,
) -> Result<HttpResponse, MailManErrResponser> {
    script_caller::call_async(req.into_inner(), &db_pool, &mq_pool).await
}
