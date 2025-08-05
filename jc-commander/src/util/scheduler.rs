use chrono::{Local, TimeZone};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use lapin::Connection;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::config::server;
use crate::model::cron_job::CronJobModel;

// 分轮任务容器
#[derive(Clone)]
struct MinuteSlot {
    remaining_sec: usize,
    job: Arc<CronJobModel>,
}

// 时轮任务容器
#[derive(Clone)]
struct HourSlot {
    remaining_min: usize,
    remaining_sec: usize,
    job: Arc<CronJobModel>,
}

pub struct TimeWheel {
    inner: Mutex<TimeWheelInner>,
    db_pool: Pool<ConnectionManager<PgConnection>>,
    mq_pool: Arc<Connection>,
}

struct TimeWheelInner {
    s_wheels: Vec<VecDeque<Arc<CronJobModel>>>,
    m_wheels: Vec<VecDeque<MinuteSlot>>,
    h_wheels: Vec<VecDeque<HourSlot>>,
    s_index: u8,
    m_index: u8,
    h_index: u8,
}

impl TimeWheel {
    pub fn new(db_pool: Pool<ConnectionManager<PgConnection>>, mq_pool: Arc<Connection>) -> Self {
        log::info!("TimeWheel init...");
        Self {
            inner: Mutex::new(TimeWheelInner {
                s_wheels: (0..60).map(|_| VecDeque::new()).collect(),
                m_wheels: (0..60).map(|_| VecDeque::new()).collect(),
                h_wheels: (0..24).map(|_| VecDeque::new()).collect(),
                s_index: 0,
                m_index: 0,
                h_index: 0,
            }),
            db_pool,
            mq_pool,
        }
    }

    pub fn add_task(&self, job: CronJobModel) -> Result<(), String> {
        let rel_sec = Local
            .from_local_datetime(&job.launch_at)
            .single()
            .expect("launch_at is not a valid local datetime")
            .timestamp()
            - Local::now().timestamp()
            + (job.frequency * job.times as i64);
        if rel_sec < 0 {
            return Err("Invalid launch time".into());
        }
        let rel_sec = rel_sec as usize;

        log::info!("new task regist in {rel_sec}s");

        let mut inner = self.inner.lock().unwrap();
        let job_arc = Arc::new(job);

        match rel_sec {
            s @ 0..=59 => {
                let target = (inner.s_index as usize + s) % 60;
                inner.s_wheels[target].push_back(job_arc);
            }
            s @ 60..=3599 => {
                let rem_m = s / 60;
                let target = (inner.m_index as usize + rem_m) % 60;
                inner.m_wheels[target].push_back(MinuteSlot {
                    remaining_sec: s % 60,
                    job: job_arc,
                });
            }
            s => {
                let rem_h = s / 3600;
                let target = (inner.h_index as usize + rem_h) % 24;
                inner.h_wheels[target].push_back(HourSlot {
                    remaining_min: (s % 3600) / 60,
                    remaining_sec: s % 60,
                    job: job_arc,
                });
            }
        }
        Ok(())
    }

    pub fn tick(&self) -> bool {
        // log::info!("ticking...");
        let mut inner = self.inner.lock().unwrap();
        inner.s_index = (inner.s_index + 1) % 60;

        if inner.s_index == 0 {
            inner.m_index = (inner.m_index + 1) % 60;
            self.cascade_minutes(&mut inner);
        }

        self.process_seconds(&mut inner)
    }

    fn cascade_minutes(&self, inner: &mut TimeWheelInner) {
        if inner.m_index == 0 {
            inner.h_index = (inner.h_index + 1) % 24;
            self.cascade_hours(inner);
        }
        self.process_minutes(inner);
    }

    fn cascade_hours(&self, inner: &mut TimeWheelInner) {
        let target = inner.h_index as usize;
        while let Some(hour_slot) = inner.h_wheels[target].pop_front() {
            let m_target = (inner.m_index as usize + hour_slot.remaining_min) % 60;
            inner.m_wheels[m_target].push_back(MinuteSlot {
                remaining_sec: hour_slot.remaining_sec,
                job: hour_slot.job,
            });
        }
    }

    fn process_seconds(&self, inner: &mut TimeWheelInner) -> bool {
        let target = inner.s_index as usize;
        let jobs: Vec<_> = inner.s_wheels[target].drain(..).collect();

        let mut conn = self.db_pool.get().expect("Failed to get DB connection");
        let mut retry_list = Vec::new();

        for job in jobs {
            match self.send_job(&job) {
                Ok(_) => {
                    if let Err(e) = CronJobModel::next_round(&job.id, &mut conn) {
                        log::error!("Failed to update job status: {e:?}");
                    }
                    retry_list.push(Arc::into_inner(job).unwrap());
                }
                Err(e) => {
                    log::error!("Failed to send job: {e:?}");
                }
            }
        }

        // for job in retry_list {
        //     log::info!("job push next round id: {}", job.id);
        //     if let Err(e) = self.add_task(job) {
        //         log::error!("Failed to requeue job: {}", e);
        //     }
        // }
        if !retry_list.is_empty() {
            log::info!(
                "job push next round ids: {:?}",
                retry_list
                    .iter()
                    .map(|job| { job.id.clone() })
                    .collect::<Vec<String>>()
            );
            return true;
        }
        false
    }

    fn process_minutes(&self, inner: &mut TimeWheelInner) {
        let target = inner.m_index as usize;
        while let Some(minute_slot) = inner.m_wheels[target].pop_front() {
            let s_target = (inner.s_index as usize + minute_slot.remaining_sec) % 60;
            inner.s_wheels[s_target].push_back(minute_slot.job);
        }
    }

    pub fn clear(&self) -> Result<MailManOk<String>, ()> {
        log::info!("cleaning TimeWheel...");
        let mut inner = self.inner.lock().unwrap();
        inner.s_wheels.iter_mut().for_each(VecDeque::clear);
        inner.m_wheels.iter_mut().for_each(VecDeque::clear);
        inner.h_wheels.iter_mut().for_each(VecDeque::clear);
        Ok(MailManOk {
            code: 200,
            key: "TimeWheel clear",
            data: None,
        })
    }

    pub fn reload_from_db(&self) -> Result<(), String> {
        let mut conn = self
            .db_pool
            .get()
            .map_err(|e| format!("DB connection failed: {e:?}"))?;

        let filter: serde_json::Map<String, serde_json::Value> =
            serde_json::from_value(serde_json::json!({"status": 1})).expect("filter build error");
        let jobs = CronJobModel::get_crons_with_filter(&filter, &mut conn)
            .map_err(|e| format!("Query failed: {e:?}"))?;

        let _ = self.clear();

        for job in jobs {
            self.add_task(job.clone())
                .map_err(|e| format!("Failed to add job {}: {}", job.id.clone(), e))?;
        }

        Ok(())
    }

    async fn send_job_async(&self, job: &CronJobModel) -> Result<(), lapin::Error> {
        let channel = self.mq_pool.create_channel().await?;
        let subsys_uuid = { server::GLOBAL_CONFIG.read().unwrap().subsys_uuid.clone() };
        let queue = {
            let prefix = server::GLOBAL_CONFIG
                .read()
                .unwrap()
                .mq_queue_prefix
                .clone();
            format!("{prefix}_async")
        };
        let payload = serde_json::json!({
            "commander": subsys_uuid,
            "host": "any",
            "id": job.id,
            "params": job.params,
            "script": job.script
        });

        channel
            .basic_publish(
                "",
                &queue,
                lapin::options::BasicPublishOptions::default(),
                serde_json::to_vec(&payload)
                    .expect("params encode Error")
                    .as_slice(),
                lapin::BasicProperties::default(),
            )
            .await?;

        Ok(())
    }

    fn send_job(
        &self,
        job: &Arc<CronJobModel>,
    ) -> Result<MailManOk<'static, String>, MailManErr<'static, String>> {
        log::info!("sending job...");
        futures::executor::block_on(self.send_job_async(job))
            .map(|_| {
                MailManOk::new(
                    200,
                    "Timer task send success",
                    Some("Message queued successfully".into()),
                )
            })
            .map_err(|e| {
                MailManErr::new(
                    500,
                    "Task sending Failed",
                    Some(format!("MQ error: {e}")),
                    1,
                )
            })
    }
}
