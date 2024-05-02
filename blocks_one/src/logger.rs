use std::fs::{File, OpenOptions};
use std::io::{Write};
use std::{result, fmt};

use async_std::sync::Mutex;

pub struct FileLogger {
    info_file: Mutex<File>,
    err_file: Mutex<File>,
}

pub trait Logger {
    async fn log(&self, level: LogLevel, msg: &str);
}



impl FileLogger {
    pub fn new(info_path: &str, err_path: &str) -> Result<Self, std::io::Error> {
        let info_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(info_path)?;

        let err_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(err_path)?;

        Ok(FileLogger { 
            info_file: Mutex::new(info_file), 
            err_file: Mutex::new(err_file) 
        })
    }
}

impl Logger for FileLogger {
    async fn log(&self, level: LogLevel, msg: &str) {
        let fmt_msg = format!("[{}] {}\n", level, msg);
        let file = match level {
            LogLevel::Info => &self.info_file,
            LogLevel::Err => &self.err_file,
        };
        let _ = file.lock().await.write_all(fmt_msg.as_bytes());
    }
}

#[derive(Debug)]
pub enum LogLevel {
    Info,
    Err,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            LogLevel::Info => "INFO",
            LogLevel::Err => "ERR",
        })
    }
}