use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
// use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::{access::*, service::*};

/// all_service api logic
pub fn all_service<'a>(
    filter: Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Vec<ServiceModel>>, MailManErr<'a, String>> {
    match ServiceModel::get_all_with_filter(&filter, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "All service info", Some(msg))),
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

/// new_service api logic
pub fn new_service<'a>(
    service_info: ServiceInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match ServiceModel::new_service(&service_info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service created", Some(msg))),
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

/// update_service api logic
pub fn update_service<'a>(
    service_info: ServiceInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match ServiceModel::update_service_by_id(&service_info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service info updated", Some(msg))),
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

/// delete_service api logic
pub fn delete_service<'a>(
    id: i32,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    let access_target: Vec<i32> =
        match AccessModel::get_access_by_sids(&vec![id], &mut pool.get().unwrap()) {
            Ok(access_info_list) => access_info_list
                .into_iter()
                .map(|access_info| access_info.id)
                .collect(),
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

    for access_id in &access_target {
        let update_info_disable = AccessInfo {
            id: Some(*access_id),
            service_id: None,
            group_id: None,
            group_access: None,
            is_enable: Some(false),
            update_time: None,
            comment: None,
        };

        match AccessModel::update_access_by_id(&update_info_disable, &mut pool.get().unwrap()) {
            Ok(_) => (),
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
        }
    }

    match ServiceModel::delete_service_by_id(id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service deleted",
            Some(format!(
                "Service table: {msg}. And disabled those access line {access_target:?}"
            )),
        )),
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
