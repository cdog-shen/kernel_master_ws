use actix_web::web;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use serde_json::{Map, Value};
use ureq;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::models::{access::*, service::*, subsys::*};

/// all_subsys api logic
pub fn all_subsys<'a>(
    filter: &Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, Vec<SubsysOutputStream>>, MailManErr<'a>> {
    match SubsysModel::get_all_with_filter(filter.clone(), &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "All Subsystem info", Some(msg))),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

/// new_subsys api logic
pub fn new_subsys<'a>(
    subsys_info: &SubsysInputStream,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    match ServiceModel::new_service(
        &ServiceInputStream {
            id: None,
            service_name: Some(format!(
                "bind_{}",
                &subsys_info.subsys_name.clone().unwrap()
            )),
            service_point: Some(format!(
                "/subsystem/{}",
                &subsys_info.subsys_name.clone().unwrap()
            )),
            is_enable: Some(1),
            create_time: None,
        },
        &mut pool.get().unwrap(),
    ) {
        Ok(msg) => MailManOk::new(200, "Subsystem bind service created", Some(msg)),
        Err(msg) => match msg.0 {
            0 => return Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => return Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    };

    match SubsysModel::new_meta(&subsys_info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Subsystem meta data created",
            Some(msg),
        )),
        Err(msg) => match msg.0 {
            0 => return Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => return Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

/// update_subsys api logic
pub fn update_subsys<'a>(
    subsys_info: &SubsysInputStream,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    match SubsysModel::update_meta_by_id(&subsys_info, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Subsystem meta data updated",
            Some(msg),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

/// delete_subsys api logic
pub fn delete_subsys<'a>(
    subsys_meta_id: u32,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a>> {
    let bind_service_id =
        match SubsysModel::get_meta_by_id(subsys_meta_id, &mut pool.get().unwrap()) {
            Ok(sub_meta) => sub_meta.relate_service.unwrap(),
            Err(msg) => match msg.0 {
                0 => return Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
                _ => return Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
            },
        };

    let access_target: Vec<u32> =
        match AccessModel::get_access_by_sids(vec![bind_service_id], &mut pool.get().unwrap()) {
            Ok(access_info_list) => access_info_list
                .into_iter()
                .filter_map(|access_info| access_info.id)
                .collect(),
            Err(msg) => match msg.0 {
                0 => return Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
                _ => return Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
            },
        };

    for access_id in &access_target {
        match AccessModel::update_access_by_id(
            &AccessInputStream {
                id: Some(*access_id),
                is_enable: Some(0),
                service_id: None,
                group_id: None,
                group_access: None,
                update_time: None,
                comment: None,
            },
            &mut pool.get().unwrap(),
        ) {
            Ok(_) => (),
            Err(msg) => match msg.0 {
                0 => return Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
                _ => return Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
            },
        }
    }

    let _ = match ServiceModel::delete_service_by_id(bind_service_id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service deleted",
            Some(format!(
                "Service table: {}. And disabled those access line {:?}",
                msg, access_target
            )),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    };

    match SubsysModel::delete_meta_by_id(subsys_meta_id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Subsystem meta data deleted",
            Some(msg),
        )),
        Err(msg) => match msg.0 {
            0 => Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
            _ => Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
        },
    }
}

pub fn call<'a>(
    subsys_name: String,
    subsys_params: serde_json::Map<String, serde_json::Value>,
    pool: &web::Data<Pool<ConnectionManager<MysqlConnection>>>,
) -> Result<MailManOk<'a, serde_json::Value>, MailManErr<'a>> {
    let target =
        match SubsysModel::get_enable_by_name(subsys_name.clone(), &mut pool.get().unwrap()) {
            Ok(subsys_info) => subsys_info,
            Err(msg) => match msg.0 {
                0 => return Err(MailManErr::new(500, "Internal Server Error", msg.1, 1)),
                _ => return Err(MailManErr::new(400, "Bad requests", msg.1, 1)),
            },
        };

    let req = ureq::post(
        format!(
            "{}/{}/{}",
            &target.url.unwrap(),
            &subsys_params["operation"],
            &subsys_params["target"]
        )
        .as_str(),
    )
    .set("Content-Type", "application/json")
    .set("Authorization", &format!("uuid {}", &target.token.unwrap()))
    .send_json(&subsys_params["data"]);

    let _resp = match req {
        Ok(resp) => {
            return Ok(MailManOk::new(
                200,
                "Subsystem call success",
                Some({
                    let resp_text = resp.into_string().unwrap();
                    match serde_json::from_str::<serde_json::Value>(&resp_text) {
                        Ok(json_value) => json_value,
                        Err(_) => {
                            serde_json::json!({
                                "data": resp_text,
                            })
                        }
                    }
                }),
            ));
        }
        Err(msg) => match msg.kind() {
            _ => {
                return Err(MailManErr::new(
                    500,
                    "Internal Server Error",
                    format!("Subsystem: {}. Error kind: {}", &subsys_name, msg),
                    1,
                ))
            }
        },
    };
}
