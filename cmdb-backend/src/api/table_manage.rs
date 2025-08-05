use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};

use share_lib::data_structure::MailManErr;

use crate::{model, service, util::err_mapping::MailManErrResponser};

// GET api/cmdb/get_all_table
// pub async fn get_all_table() -> Result<HttpResponse, actix_web::Error> {
//     let data = serde_json::json!(["lighthouse",]);
//     Ok(HttpResponse::Ok().json(data))
// }

// POST api/table/query
pub async fn query_table(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let data = data.into_inner();
    let table_name = data.get("table").unwrap().as_str().unwrap();
    let query = serde_json::from_value(
        data.get("query")
            .ok_or(MailManErrResponser::mapping_from_mme(MailManErr {
                code: 400,
                key: "Bad request",
                msg: Some("Query operation must have `query` field".to_string()),
                level: 1,
            }))?
            .clone(),
    )
    .map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr {
            code: 400,
            key: "Bad request",
            msg: Some(e.to_string()),
            level: 1,
        })
    })?;

    match table_name {
        "lighthouse" => match service::lighthouse::instance_service::get_all(query, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        "cloudserver_instance" => {
            match service::cloudserver::instance_service::get_all(query, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        "logservice_topic" => match service::logservice::topic_service::get_all(query, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        _ => Err(MailManErrResponser::mapping_from_mme(MailManErr {
            code: 400,
            key: "Bad request",
            msg: Some("No such table".to_string()),
            level: 1,
        })),
    }
}

// POST api/table/new
pub async fn new_table(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let data = data.into_inner();
    let table_name = data.get("table").unwrap().as_str().unwrap();
    let new = serde_json::from_value(
        data.get("new")
            .ok_or(MailManErrResponser::mapping_from_mme(MailManErr {
                code: 400,
                key: "Bad request",
                msg: Some("Creat operation must have `new` field".to_string()),
                level: 1,
            }))?
            .clone(),
    )
    .map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr {
            code: 400,
            key: "Bad request",
            msg: Some(e.to_string()),
            level: 1,
        })
    })?;

    match table_name {
        "lighthouse" => {
            let new = model::lighthouse::instance::LightEcsInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(
                    400,
                    "Bad Request",
                    Some(e),
                    1,
                ))
            })?;
            match service::lighthouse::instance_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        "cloudserver_instance" => {
            let new = model::cloudserver::instance::CloudserverInstanceInfo::from_map(new)
                .map_err(|e| {
                    MailManErrResponser::mapping_from_mme(MailManErr::new(
                        400,
                        "Bad Request",
                        Some(e),
                        1,
                    ))
                })?;
            match service::cloudserver::instance_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        "logservice_topic" => {
            let new =
                model::logservice::topic::LogServiceTopicInfo::from_map(new).map_err(|e| {
                    MailManErrResponser::mapping_from_mme(MailManErr::new(
                        400,
                        "Bad Request",
                        Some(e),
                        1,
                    ))
                })?;
            match service::logservice::topic_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        _ => Err(MailManErrResponser::mapping_from_mme(MailManErr {
            code: 400,
            key: "Bad request",
            msg: Some("No such table".to_string()),
            level: 1,
        })),
    }
}

// POST api/table/update
pub async fn update_table(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let data = data.into_inner();
    let table_name = data.get("table").unwrap().as_str().unwrap();
    let update = serde_json::from_value(
        data.get("update")
            .ok_or(MailManErrResponser::mapping_from_mme(MailManErr {
                code: 400,
                key: "Bad request",
                msg: Some("Update operation must have `update` field".to_string()),
                level: 1,
            }))?
            .clone(),
    )
    .map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr {
            code: 400,
            key: "Bad request",
            msg: Some(e.to_string()),
            level: 1,
        })
    })?;

    match table_name {
        "lighthouse" => {
            let update =
                model::lighthouse::instance::LightEcsInfo::from_map(update).map_err(|e| {
                    MailManErrResponser::mapping_from_mme(MailManErr::new(
                        400,
                        "Bad Request",
                        Some(e),
                        1,
                    ))
                })?;
            match service::lighthouse::instance_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        "cloudserver_instance" => {
            let update = model::cloudserver::instance::CloudserverInstanceInfo::from_map(update)
                .map_err(|e| {
                    MailManErrResponser::mapping_from_mme(MailManErr::new(
                        400,
                        "Bad Request",
                        Some(e),
                        1,
                    ))
                })?;
            match service::cloudserver::instance_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        "logservice_topic" => {
            let update =
                model::logservice::topic::LogServiceTopicInfo::from_map(update).map_err(|e| {
                    MailManErrResponser::mapping_from_mme(MailManErr::new(
                        400,
                        "Bad Request",
                        Some(e),
                        1,
                    ))
                })?;
            match service::logservice::topic_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        _ => Err(MailManErrResponser::mapping_from_mme(MailManErr {
            code: 400,
            key: "Bad request",
            msg: Some("No such table".to_string()),
            level: 1,
        })),
    }
}

// DELETE api/table/delete
pub async fn delete_table(
    data: web::Json<Map<String, Value>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, MailManErrResponser> {
    let data = data.into_inner();
    let table_name = data.get("table").unwrap().as_str().unwrap();
    let id = serde_json::from_value(
        data.get("delete")
            .ok_or(MailManErrResponser::mapping_from_mme(MailManErr {
                code: 400,
                key: "Bad request",
                msg: Some("Delete operation must have `Delete.id` field".to_string()),
                level: 1,
            }))?
            .get("id")
            .ok_or(MailManErrResponser::mapping_from_mme(MailManErr {
                code: 400,
                key: "Bad request",
                msg: Some("Delete operation must have `Delete.id` field".to_string()),
                level: 1,
            }))?
            .clone(),
    )
    .map_err(|e| {
        MailManErrResponser::mapping_from_mme(MailManErr {
            code: 400,
            key: "Bad request",
            msg: Some(e.to_string()),
            level: 1,
        })
    })?;

    match table_name {
        "lighthouse" => match service::lighthouse::instance_service::delete_table(id, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },

        "cloudserver_instance" => {
            match service::cloudserver::instance_service::delete_table(id, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        "logservice_topic" => match service::logservice::topic_service::delete_table(id, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },

        _ => Err(MailManErrResponser::mapping_from_mme(MailManErr {
            code: 400,
            key: "Bad request",
            msg: Some("No such table".to_string()),
            level: 1,
        })),
    }
}
