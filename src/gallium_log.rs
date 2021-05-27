extern crate log;
use log::{Record, Level, LevelFilter, Metadata, SetLoggerError};

pub struct GalliumLog;

impl log::Log for GalliumLog {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() != Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mark = match record.level() {
                Level::Error => "!!!",
                Level::Warn => "!",
                Level::Info => ">",
                Level::Debug => "+",
                Level::Trace => "-",
            };
            println!("[{}] {}", mark, record.args());
        }
    }

    fn flush(&self) { }
}

static MY_LOGGER: GalliumLog = GalliumLog;
impl GalliumLog {
    pub fn init() -> Result<(), SetLoggerError> {
        log::set_logger(&MY_LOGGER)?;
        log::set_max_level(LevelFilter::Debug);
        Ok(())
    }
}
