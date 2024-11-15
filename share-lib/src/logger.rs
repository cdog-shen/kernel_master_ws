// src/logger.rs
use log::Level;
use log::{LevelFilter, Log, Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::{Result as IoResult, Write};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

pub struct FileLogger {
    file: Arc<Mutex<File>>,
}

impl FileLogger {
    pub fn new(log_file: &str) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .expect("Unable to open log file");

        FileLogger {
            file: Arc::new(Mutex::new(file)),
        }
    }

    pub fn log(&self, message: &str) -> IoResult<()> {
        let mut file = self.file.lock().unwrap();
        writeln!(file, "{}", message)
    }
}

// custom logger
pub struct CombinedLogger {
    file_logger: FileLogger,
}

impl CombinedLogger {
    pub fn new(log_file: &str) -> Self {
        let file_logger = FileLogger::new(log_file);
        CombinedLogger { file_logger }
    }
}

impl Log for CombinedLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info // 可以根据需要调整级别
    }

    fn log(&self, record: &Record) {
        let message = format!("{} - {}", record.level(), record.args());

        // output
        if let Err(e) = self.file_logger.log(&message) {
            eprintln!("Failed to write to log file: {}", e);
        }

        // print
        println!("{}", message);
    }

    fn flush(&self) {}
}

// 定义日志初始化函数
pub fn init_logger(log_level: &str) {
    // 读取自定义环境变量
    let log_level = log_level.to_string();

    let combined_logger = CombinedLogger::new("logs/watchman.log");

    log::set_boxed_logger(Box::new(combined_logger)).unwrap();
    log::set_max_level(LevelFilter::from_str(&log_level).unwrap_or(LevelFilter::Info));
}

// 宏用于简化日志记录
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
