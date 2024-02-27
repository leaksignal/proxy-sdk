use log::{Level, LevelFilter};

use crate::hostcalls::{self, LogLevel};
use std::borrow::Cow;
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};

struct Logger;

static LOGGER: Logger = Logger;
static INITIALIZED: AtomicBool = AtomicBool::new(false);

impl From<Level> for LogLevel {
    fn from(val: Level) -> Self {
        match val {
            Level::Error => LogLevel::Error,
            Level::Warn => LogLevel::Warn,
            Level::Info => LogLevel::Info,
            Level::Debug => LogLevel::Debug,
            Level::Trace => LogLevel::Trace,
        }
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(val: LogLevel) -> Self {
        match val {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Critical => LevelFilter::Off,
        }
    }
}

/// Sets the log level filter and installs a panic hook to log out panics.
pub fn set_log_level(level: Level) {
    if !INITIALIZED.load(Ordering::Relaxed) {
        log::set_logger(&LOGGER).unwrap();
        panic::set_hook(Box::new(|panic_info| {
            hostcalls::log(LogLevel::Critical, &panic_info.to_string()).unwrap();
        }));
        INITIALIZED.store(true, Ordering::Relaxed);
    }
    LOGGER.set_log_level(level.into());
}

impl Logger {
    pub fn set_log_level(&self, level: LogLevel) {
        log::set_max_level(level.into());
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let args = record.args();
            let message = match args.as_str() {
                Some(v) => Cow::Borrowed(v),
                None => Cow::Owned(args.to_string()),
            };
            hostcalls::log(record.level().into(), &message).unwrap();
        }
    }

    fn flush(&self) {}
}
