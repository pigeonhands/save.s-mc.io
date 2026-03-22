#![deny(missing_docs)]

//! Logger implementation for Cloudflare Workers.
//! Bridges the [`log`](https://crates.io/crates/log) ecosystem to Cloudflare Worker.

use log::{Level, Metadata, Record, set_max_level};
use worker::{Date, console_debug, console_error, console_log, console_warn};

use log::set_logger;

use std::str::FromStr;

static WORKER_LOGGER: Logger = Logger {};

/// Main logger struct
#[derive(Debug)]
pub struct Logger {}

impl Logger {
    /// Initialize the logger with a string
    pub fn new<S: AsRef<str>>(init_string: S) -> Self {
        {
            let level = Level::from_str(init_string.as_ref());
            if let Err(ref e) = level {
                console_debug!("Failed to parse log level string, fallback to info: {}", e);
            }
            set_max_level(level.unwrap_or(Level::Info).to_level_filter());
        }
        Logger {}
    }

    /// Set the logger instance as the main logger
    pub fn set_logger(self) {
        let result = set_logger(&WORKER_LOGGER);
        if let Err(e) = result {
            console_error!("Logger installation failed: {}", e);
        }
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let target = match (record.file(), record.line()) {
            (Some(file), Some(line)) => format!("{}:{}", file, line),
            _ => record.target().to_string(),
        };
        let level = record.level().to_string();
        let prompt = format!(
            "[{time} {level} {target}]",
            time = Date::now().to_string(),
            level = level,
            target = target,
        );
        match record.level() {
            Level::Debug => console_debug!("{} {}", prompt, record.args()),
            Level::Error => console_error!("{} {}", prompt, record.args()),
            Level::Warn => console_warn!("{} {}", prompt, record.args()),
            _ => console_log!("{} {}", prompt, record.args()),
        }
    }

    fn flush(&self) {}
}

/// Initialize and install a logger with a `log::Level`
pub fn init_with_level(level: &Level) {
    Logger::new(level.as_str()).set_logger();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
