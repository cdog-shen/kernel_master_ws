use actix_web::{HttpResponse, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};

use share_lib::data_structure::MailManErr;

use crate::{model, service, util::err_mapping::MailManErrResponser};

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
        // === cmdb native tables ===
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
        "cloud_account" => match service::km::cloud_account_service::get_all(query, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        "job_log" => match service::km::job_log_service::get_all(query, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        "cron_job" => match service::km::cron_job_service::get_all(query, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        "cloudstorage_bucket" => {
            match service::cloudstorage::bucket_service::get_all(query, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        // === watchman tables (feature-gated) ===
        #[cfg(feature = "full-schema")]
        "access_table" => match service::watchman::access_service::get_all(query, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        #[cfg(feature = "full-schema")]
        "group_table" => match service::watchman::group_service::get_all(query, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        #[cfg(feature = "full-schema")]
        "service_table" => match service::watchman::service_service::get_all(query, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        #[cfg(feature = "full-schema")]
        "subsystem_table" => {
            match service::watchman::subsystem_service::get_all(query, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "token_table" => match service::watchman::token_service::get_all(query, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        #[cfg(feature = "full-schema")]
        "user_table" => match service::watchman::user_service::get_all(query, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        #[cfg(feature = "full-schema")]
        "webhook_table" => match service::watchman::webhook_service::get_all(query, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },

        // === yell tables (feature-gated) ===
        #[cfg(feature = "full-schema")]
        "channel_configs" => {
            match service::yell::channel_config_service::get_all(query, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "notification_records" => {
            match service::yell::notification_record_service::get_all(query, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "notification_templates" => {
            match service::yell::notification_template_service::get_all(query, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "notification_groups" => {
            match service::yell::notification_group_service::get_all(query, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "notification_group_members" => {
            match service::yell::notification_group_member_service::get_all(query, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        _ => Err(MailManErrResponser::mapping_from_mme(MailManErr {
            code: 400,
            key: "Bad request",
            msg: Some(format!("No such table: {}", table_name)),
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
        // === cmdb native tables ===
        "lighthouse" => {
            let new = model::lighthouse::instance::LightEcsInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::lighthouse::instance_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        "cloudserver_instance" => {
            let new = model::cloudserver::instance::CloudserverInstanceInfo::from_map(new)
                .map_err(|e| {
                    MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
                })?;
            match service::cloudserver::instance_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        "logservice_topic" => {
            let new = model::logservice::topic::LogServiceTopicInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::logservice::topic_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        "cloud_account" => {
            let new = model::km::cloud_account::CloudAccountInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::km::cloud_account_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        "job_log" => {
            let new = model::km::job_log::JobLogInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::km::job_log_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        "cron_job" => {
            let new = model::km::cron_job::CronJobInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::km::cron_job_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        "cloudstorage_bucket" => {
            let new = model::cloudstorage::bucket::BucketInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::cloudstorage::bucket_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        // === watchman tables (feature-gated) ===
        #[cfg(feature = "full-schema")]
        "access_table" => {
            let new = model::watchman::access::AccessInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::watchman::access_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "group_table" => {
            let new = model::watchman::group::GroupInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::watchman::group_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "service_table" => {
            let new = model::watchman::service::ServiceInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::watchman::service_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "subsystem_table" => {
            let new = model::watchman::subsystem::SubsystemInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::watchman::subsystem_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "token_table" => {
            let new = model::watchman::token::TokenInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::watchman::token_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "user_table" => {
            let new = model::watchman::user::UserInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::watchman::user_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "webhook_table" => {
            let new = model::watchman::webhook::WebhookInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::watchman::webhook_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        // === yell tables (feature-gated) ===
        #[cfg(feature = "full-schema")]
        "channel_configs" => {
            let new = model::yell::channel_config::ChannelConfigInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::yell::channel_config_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "notification_records" => {
            let new = model::yell::notification_record::NotificationRecordInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::yell::notification_record_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "notification_templates" => {
            let new = model::yell::notification_template::NotificationTemplateInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::yell::notification_template_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "notification_groups" => {
            let new = model::yell::notification_group::NotificationGroupInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::yell::notification_group_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "notification_group_members" => {
            let new = model::yell::notification_group_member::NotificationGroupMemberInfo::from_map(new).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::yell::notification_group_member_service::new_table(new, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        _ => Err(MailManErrResponser::mapping_from_mme(MailManErr {
            code: 400,
            key: "Bad request",
            msg: Some(format!("No such table: {}", table_name)),
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
        // === cmdb native tables ===
        "lighthouse" => {
            let update = model::lighthouse::instance::LightEcsInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::lighthouse::instance_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        "cloudserver_instance" => {
            let update = model::cloudserver::instance::CloudserverInstanceInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::cloudserver::instance_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        "logservice_topic" => {
            let update = model::logservice::topic::LogServiceTopicInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::logservice::topic_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        "cloud_account" => {
            let update = model::km::cloud_account::CloudAccountInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::km::cloud_account_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        "job_log" => {
            let update = model::km::job_log::JobLogInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::km::job_log_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        "cron_job" => {
            let update = model::km::cron_job::CronJobInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::km::cron_job_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        "cloudstorage_bucket" => {
            let update = model::cloudstorage::bucket::BucketInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::cloudstorage::bucket_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        // === watchman tables (feature-gated) ===
        #[cfg(feature = "full-schema")]
        "access_table" => {
            let update = model::watchman::access::AccessInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::watchman::access_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "group_table" => {
            let update = model::watchman::group::GroupInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::watchman::group_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "service_table" => {
            let update = model::watchman::service::ServiceInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::watchman::service_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "subsystem_table" => {
            let update = model::watchman::subsystem::SubsystemInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::watchman::subsystem_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "token_table" => {
            let update = model::watchman::token::TokenInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::watchman::token_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "user_table" => {
            let update = model::watchman::user::UserInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::watchman::user_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "webhook_table" => {
            let update = model::watchman::webhook::WebhookInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::watchman::webhook_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        // === yell tables (feature-gated) ===
        #[cfg(feature = "full-schema")]
        "channel_configs" => {
            let update = model::yell::channel_config::ChannelConfigInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::yell::channel_config_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "notification_records" => {
            let update = model::yell::notification_record::NotificationRecordInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::yell::notification_record_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "notification_templates" => {
            let update = model::yell::notification_template::NotificationTemplateInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::yell::notification_template_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "notification_groups" => {
            let update = model::yell::notification_group::NotificationGroupInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::yell::notification_group_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "notification_group_members" => {
            let update = model::yell::notification_group_member::NotificationGroupMemberInfo::from_map(update).map_err(|e| {
                MailManErrResponser::mapping_from_mme(MailManErr::new(400, "Bad Request", Some(e), 1))
            })?;
            match service::yell::notification_group_member_service::update_table(update, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        _ => Err(MailManErrResponser::mapping_from_mme(MailManErr {
            code: 400,
            key: "Bad request",
            msg: Some(format!("No such table: {}", table_name)),
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
        // === cmdb native tables ===
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
        "cloud_account" => match service::km::cloud_account_service::delete_table(id as i32, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        "job_log" => match service::km::job_log_service::delete_table(id.to_string(), &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        "cron_job" => match service::km::cron_job_service::delete_table(id.to_string(), &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        "cloudstorage_bucket" => {
            match service::cloudstorage::bucket_service::delete_table(id, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        // === watchman tables (feature-gated) ===
        #[cfg(feature = "full-schema")]
        "access_table" => match service::watchman::access_service::delete_table(id as i32, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        #[cfg(feature = "full-schema")]
        "group_table" => match service::watchman::group_service::delete_table(id as i32, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        #[cfg(feature = "full-schema")]
        "service_table" => match service::watchman::service_service::delete_table(id as i32, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        #[cfg(feature = "full-schema")]
        "subsystem_table" => {
            match service::watchman::subsystem_service::delete_table(id as i32, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "token_table" => match service::watchman::token_service::delete_table(id.to_string(), &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        #[cfg(feature = "full-schema")]
        "user_table" => match service::watchman::user_service::delete_table(id as i32, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },
        #[cfg(feature = "full-schema")]
        "webhook_table" => match service::watchman::webhook_service::delete_table(id as i32, &pool) {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
        },

        // === yell tables (feature-gated) ===
        #[cfg(feature = "full-schema")]
        "channel_configs" => {
            match service::yell::channel_config_service::delete_table(id as i32, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "notification_records" => {
            match service::yell::notification_record_service::delete_table(id as i32, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "notification_templates" => {
            match service::yell::notification_template_service::delete_table(id as i32, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "notification_groups" => {
            match service::yell::notification_group_service::delete_table(id as i32, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }
        #[cfg(feature = "full-schema")]
        "notification_group_members" => {
            match service::yell::notification_group_member_service::delete_table(id as i32, &pool) {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(err) => Err(MailManErrResponser::mapping_from_mme(err)),
            }
        }

        _ => Err(MailManErrResponser::mapping_from_mme(MailManErr {
            code: 400,
            key: "Bad request",
            msg: Some(format!("No such table: {}", table_name)),
            level: 1,
        })),
    }
}
