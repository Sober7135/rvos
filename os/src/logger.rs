// extern crate alloc;
use crate::println;

struct Logger;

impl log::Log for Logger {
    #[allow(unused_variables)]
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let color = match record.level() {
            log::Level::Trace => 90, // bright black
            log::Level::Info => 34,  // blue
            log::Level::Debug => 32, // green
            log::Level::Warn => 93,  // bright yellow
            log::Level::Error => 31, // red
        };
        println!(
            "{}:{} \u{1B}[0;{}m[{}]\u{1B}[0m {}",
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            color,
            record.level(),
            record.args()
        );
    }

    fn flush(&self) {}
}

pub(crate) fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();
    let env = option_env!("LOG").unwrap_or("TRACE");

    log::set_max_level(match env {
        "TRACE" => log::LevelFilter::Trace,
        "DEBUG" => log::LevelFilter::Debug,
        "INFO" => log::LevelFilter::Info,
        "WARN" => log::LevelFilter::Warn,
        "ERROR" => log::LevelFilter::Error,
        _ => log::LevelFilter::Off,
    })
}
