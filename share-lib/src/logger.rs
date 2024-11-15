// src/logger.rs
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::{Result as IoResult, Write};
use std::sync::{Arc, Mutex};
use std::str::FromStr;

pub struct ShareLogger {
    pub file: Arc<Mutex<File>>,
    pub level: Level,  // 使用 log crate 的 Level 类型
}

impl ShareLogger {
    // 构造函数，将文件路径和日志级别传入
    pub fn new(log_file: &str, log_level: &str) -> Self {
        // 打开日志文件
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .expect("Unable to open log file");

        // 解析日志级别字符串并转化为对应的 Level 枚举
        let level = match log_level.to_uppercase().as_str() {
            "ERROR" => Level::Error,
            "INFO" => Level::Info,
            "WARN" => Level::Warn,
            "TRACE" => Level::Trace,
            _ => Level::Debug,
        };

        ShareLogger {
            file: Arc::new(Mutex::new(file)),
            level,
        }
    }

    // 将日志消息写入文件
    fn log_to_file(&self, message: &str) -> IoResult<()> {
        let mut file = self.file.lock().unwrap();
        writeln!(file, "{}", message)
    }
}

impl Log for ShareLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        let message = format!("{} - {}", record.level(), record.args());

        // 写入到文件
        if let Err(e) = self.log_to_file(&message) {
            eprintln!("Failed to write to log file: {}", e);
        }

        // 控制台输出
        println!("{}", message);
    }

    fn flush(&self) {}
}


// logger init
// pub fn init_logger(log_level: &str) {
//     // read level
//     let log_level = log_level.to_string();

//     let combined_logger = CombinedLogger::new("logs/watchman.log");

//     log::set_boxed_logger(Box::new(combined_logger)).unwrap();
//     log::set_max_level(LevelFilter::from_str(&log_level).unwrap_or(LevelFilter::Info));
// }

// log macro
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => (info!($($arg)*));
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => (warn!($($arg)*));
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => (error!($($arg)*));
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => (debug!($($arg)*));
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => (trace!($($arg)*));
}
