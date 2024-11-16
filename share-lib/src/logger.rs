// src/logger.rs
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::{Result as IoResult, Write};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

pub struct ShareLogger {
    pub file: Arc<Mutex<File>>, // a Mutex file ptr
    pub level: Level,           // log level params
}

impl ShareLogger {
    pub fn new(log_file: &str, log_level: &str) -> Self {
        // open the log file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .expect("Unable to open log file");

        // trans log level config into Level const
        let level = match log_level.to_uppercase().as_str() {
            "ERROR" => Level::Error,
            "WARN" => Level::Warn,
            "INFO" => Level::Info,
            "DEBUG" => Level::Debug,
            _ => Level::Trace,
        };

        ShareLogger {
            file: Arc::new(Mutex::new(file)),
            level,
        }
    }

    // write log into file
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

        // write into file
        if let Err(e) = self.log_to_file(&message) {
            eprintln!("Failed to write to log file: {}", e);
        }

        // console log output
        println!("{}", message);
    }

    fn flush(&self) {}
}

// logger init
pub fn init_logger(log_path: &str, log_level_str: &str) {
    let logger_obj = ShareLogger::new(log_path, log_level_str);
    log::set_boxed_logger(Box::new(logger_obj)).unwrap();
    log::set_max_level(LevelFilter::from_str(log_level_str).unwrap_or(LevelFilter::Info));
}

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
