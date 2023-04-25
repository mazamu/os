//! Global logger

// use log::{self, Level, LevelFilter, Log, Metadata, Record};
use log::*;

/// a simple logger
struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let color = match record.level() {
            Level::Error => 31, // Red
            Level::Warn => 93,  // BrightYellow
            Level::Info => 34,  // Blue
            Level::Debug => 32, // Green
            Level::Trace => 92, // BrightBlack
        };
        println!(
            "\u{1B}[{}m[{:>5}] {}\u{1B}[0m",
            color,
            record.level(),
            record.args(),
        );
    }
    fn flush(&self) {}
}

/// initiate logger
pub fn init() {
    static LOGGER: SimpleLogger = SimpleLogger;
    static LOG: isize = 1;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("ERROR") => LevelFilter::Error,
        Some("WARN") => LevelFilter::Warn,
        Some("INFO") => LevelFilter::Info,
        Some("DEBUG") => LevelFilter::Debug,
        Some("TRACE") => LevelFilter::Trace,
        _ => LevelFilter::Off,
    });
}

// #[macro_export]
// macro_rules! log {
//     // log!(target: "my_target", Level::Info; key1 = 42, key2 = true; "a {} event", "log");
//     (target: $target:expr, $lvl:expr, $($key:tt = $value:expr),+; $($arg:tt)+) => ({
//         let lvl = $lvl;
//         if lvl <= $crate::STATIC_MAX_LEVEL && lvl <= $crate::max_level() {
//             $crate::__private_api_log(
//                 __log_format_args!($($arg)+),
//                 lvl,
//                 &($target, __log_module_path!(), __log_file!(), __log_line!()),
//                 $crate::__private_api::Option::Some(&[$((__log_key!($key), &$value)),+])
//             );
//         }
//     });

//     // log!(target: "my_target", Level::Info; "a {} event", "log");
//     (target: $target:expr, $lvl:expr, $($arg:tt)+) => ({
//         let lvl = $lvl;
//         if lvl <= $crate::STATIC_MAX_LEVEL && lvl <= $crate::max_level() {
//             $crate::__private_api_log(
//                 __log_format_args!($($arg)+),
//                 lvl,
//                 &($target, __log_module_path!(), __log_file!(), __log_line!()),
//                 $crate::__private_api::Option::None,
//             );
//         }
//     });

//     // log!(Level::Info, "a log event")
//     ($lvl:expr, $($arg:tt)+) => (log!(target: __log_module_path!(), $lvl, $($arg)+));
// }