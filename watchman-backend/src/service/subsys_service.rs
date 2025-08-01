use actix_web::web;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};
use ureq;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::service::ServiceInfo;
use crate::model::{access::*, service::*, subsys::*};

/// all_subsys api logic
pub fn all_subsys<'a>(
    filter: Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Vec<SubsysModel>>, MailManErr<'a, String>> {
    match SubsysModel::get_all_with_filter(&filter, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "All Subsystem info", Some(msg))),
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

/// new_subsys api logic
pub fn new_subsys<'a>(
    subsys: SubsysInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    let subsys_name = &subsys.subsys_name.clone().ok_or_else(|| {
        MailManErr::new(
            400,
            "Bad request",
            Some("CAN NOT find `subsys_name` field".to_string()),
            1,
        )
    })?;

    // let bind_service_info: Map<String, Value> = serde_json::from_value(serde_json::json!({
    //     "service_name": Some(format!(
    //         "bind_{}",
    //         &subsys_name
    //     )),
    //     "nick_name": Some(format!(
    //         "subsystem_{}",
    //         &subsys_name
    //     )),
    //     "service_point": Some(format!(
    //         "/api/subsystem_call/{}",
    //         &subsys_name
    //     )),
    //     "is_enable": Some(1),
    // }))
    // .unwrap();

    let bind_service_info = ServiceInfo {
        id: None,
        service_name: Some(format!("bind_{subsys_name}")),
        nick_name: Some(format!("bind_{subsys_name}")),
        service_point: Some(format!("/api/subsystem_call/{}", &subsys_name)),
        is_enable: Some(true),
        update_time: Some(chrono::Local::now().naive_local()),
    };

    match ServiceModel::new_service(&bind_service_info, &mut pool.get().unwrap()) {
        Ok(msg) => MailManOk::new(200, "Subsystem bind service created", Some(msg)),
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

    match SubsysModel::new_meta(&subsys, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Subsystem meta data created",
            Some(msg),
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

/// update_subsys api logic
pub fn update_subsys<'a>(
    subsys: SubsysInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match SubsysModel::update_meta_by_id(&subsys, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Subsystem meta data updated",
            Some(msg),
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

/// delete_subsys api logic
pub fn delete_subsys<'a>(
    id: i32,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    let bind_service_id = match SubsysModel::get_meta_by_id(id, &mut pool.get().unwrap()) {
        Ok(sub_meta) => sub_meta.relate_service_id,
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

    let access_target: Vec<i32> =
        match AccessModel::get_access_by_sids(&vec![bind_service_id], &mut pool.get().unwrap()) {
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

    let _ = match ServiceModel::delete_service_by_id(bind_service_id, &mut pool.get().unwrap()) {
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
    };

    match SubsysModel::delete_meta_by_id(id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Subsystem meta data deleted",
            Some(msg),
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

pub fn call<'a>(
    subsys_name: String,
    subsys_params: serde_json::Map<String, serde_json::Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, serde_json::Value>, MailManErr<'a, String>> {
    let target = match SubsysModel::get_enable_by_name(&subsys_name, &mut pool.get().unwrap()) {
        Ok(subsys_info) => subsys_info,
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

    let req = ureq::post(
        format!(
            "{}/{}/{}",
            &target.url,
            &subsys_params["target"].as_str().unwrap(),
            &subsys_params["operation"].as_str().unwrap(),
        )
        .as_str(),
    )
    .header("Content-Type", "application/json")
    .header("Authorization", &format!("uuid {}", &target.token))
    .header("Connection", "close")
    .send(serde_json::to_string(&subsys_params["data"]).unwrap());

    match req {
        Ok(resp) => Ok(MailManOk::new(
            200,
            "Subsystem call success",
            Some(serde_json::from_str(&resp.into_body().read_to_string().unwrap()).unwrap()),
        )),
        Err(msg) => Err(MailManErr::new(
            500,
            "Internal Server Error",
            Some(format!("Subsystem: {}. Error: {}", &subsys_name, msg)),
            1,
        )),
    }
}
