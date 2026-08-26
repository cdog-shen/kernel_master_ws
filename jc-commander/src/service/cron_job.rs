use std::sync::Mutex;

use actix_web::web;
use diesel::{
    Connection, PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde_json::{Map, Value};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::{
    cron_job::*,
    job_log::{JobLogInfo, JobLogModel},
};

// get cron by filter
pub async fn get_all<'a>(
    filter: Map<String, Value>,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, Vec<CronJobModel>>, MailManErr<'a, String>> {
    match CronJobModel::get_crons_with_filter(&filter, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(200, "Service: All Cron", Some(msg))),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(400, "Service: All Cron", Some(msg.1), 1)),
            _ => Err(MailManErr::new(500, "Service: All Cron", Some(msg.1), 1)),
        },
    }
}

// new cron job
//
// 接收清洗后的请求数据（必填字段已由 handler 校验），本层负责实体组装：
// 生成任务 uuid、固定 exec_type="cron"、从 GLOBAL_CONFIG 取 commander，
// 组装 CronJobInfo / JobLogInfo 双实体；
// cron_job 与 job_log 双表写入包在同一事务中，任一失败即整体回滚
pub async fn new<'a>(
    info: CronJobInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    let id = uuid::Uuid::new_v4().to_string();

    let cron = CronJobInfo {
        id: Some(id.clone()),
        script: info.script.clone(),
        frequency: info.frequency,
        times: info.times,
        params: info.params.clone(),
        comment: Some(info.comment.clone().unwrap_or(String::new())),
        is_enable: Some(info.is_enable.unwrap_or(false)),
        launch_at: info.launch_at,
        update_time: info.update_time,
    };

    let log = JobLogInfo {
        id: Some(id.clone()),
        script: info.script.clone(),
        exec_type: Some("cron".to_string()),
        commander: Some(
            crate::config::server::GLOBAL_CONFIG
                .read()
                .unwrap()
                .subsys_uuid
                .clone(),
        ),
        worker: Some(String::new()),
        status: Some(0),
        params: info.params.clone(),
        result: Some("{}".to_string()),
        finish_time: Some(String::new()),
        update_time: info.update_time,
        comment: Some(info.comment.clone().unwrap_or(String::new())),
    };

    let mut conn = pool.get().unwrap();

    // model 层错误是 (u8, String)，事务闭包内用 RollbackTransaction 携带回滚信号，
    // 真实错误经 captured 带出
    let mut captured: Option<(u8, String)> = None;
    let tx_result = conn.transaction::<usize, diesel::result::Error, _>(|tx| {
        match CronJobModel::new_cron(&cron, tx) {
            Ok(cron_lines) => match JobLogModel::new_log(&log, tx) {
                Ok(log_lines) => Ok(cron_lines + log_lines),
                Err(e) => {
                    captured = Some(e);
                    Err(diesel::result::Error::RollbackTransaction)
                }
            },
            Err(e) => {
                captured = Some(e);
                Err(diesel::result::Error::RollbackTransaction)
            }
        }
    });

    match tx_result {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: New Cron",
            Some(format!("Line changed: {msg}")),
        )),
        Err(e) => match captured {
            Some(msg) => match msg.0 {
                1 => Err(MailManErr::new(400, "Service: New Cron", Some(msg.1), 1)),
                _ => Err(MailManErr::new(500, "Service: New Cron", Some(msg.1), 1)),
            },
            // 非业务错误的 diesel 事务失败（如连接中断）
            None => Err(MailManErr::new(
                500,
                "Service: New Cron",
                Some(format!("Transaction Error: {e}")),
                1,
            )),
        },
    }
}

// update cron job
pub async fn update<'a>(
    cron: CronJobInfo,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match CronJobModel::update_cron(&cron, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Update Cron",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(400, "Service: Update Cron", Some(msg.1), 1)),
            _ => Err(MailManErr::new(500, "Service: Update Cron", Some(msg.1), 1)),
        },
    }
}

// delete job log
pub async fn delete<'a>(
    id: String,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, String>, MailManErr<'a, String>> {
    match CronJobModel::delete_cron(&id, &mut pool.get().unwrap()) {
        Ok(msg) => Ok(MailManOk::new(
            200,
            "Service: Delete Cron",
            Some(format!("Line changed: {msg}")),
        )),
        Err(msg) => match msg.0 {
            1 => Err(MailManErr::new(400, "Service: Delete Cron", Some(msg.1), 1)),
            _ => Err(MailManErr::new(500, "Service: Delete Cron", Some(msg.1), 1)),
        },
    }
}

// refresh TimeWheel scheduler
pub async fn refresh(
    flush_flag: &web::Data<Mutex<bool>>,
) -> Result<MailManOk<'static, Value>, MailManErr<'static, String>> {
    let mut i = flush_flag.lock().unwrap();
    *i = true;

    Ok(MailManOk::new(
        200,
        "Service: Refresh Timing Wheel",
        Some(serde_json::json!({
            "status": "ok",
            "time": chrono::Local::now()
        })),
    ))
}
