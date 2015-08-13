extern crate log;
use log::{LogRecord, LogLevel, LogLevelFilter, LogMetadata, SetLoggerError};

pub struct GalliumLog;

impl log::Log for GalliumLog {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() != LogLevel::Trace
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            let mark = match record.level() {
                LogLevel::Error => "!!!",
                LogLevel::Warn => "!",
                LogLevel::Info => ">",
                LogLevel::Debug => "+",
                LogLevel::Trace => "-",
            };
            println!("[{}] {}", mark, record.args());
        }
    }
}

impl GalliumLog {
    pub fn init() -> Result<(), SetLoggerError> {
        log::set_logger(|max_log_level| {
            max_log_level.set(LogLevelFilter::Debug);
            Box::new(GalliumLog)
        })
    }
}
