use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use share_lib::err_mapping::MailManErrResponser;

use crate::{api::filter, service::manage_service};

// ==================== 通知记录查询 API ====================

/// GET /api/record/get - 获取通知记录
pub async fn get_records(
    query: web::Query<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    match manage_service::get_records(filter::clean_record_filter(query.into_inner()), &pool).await
    {
        Ok(data) => Ok(
            HttpResponse::Ok().json(share_lib::data_structure::MailManOk::new(
                200,
                "Records fetched",
                Some(data),
            )),
        ),
        Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
    }
}
