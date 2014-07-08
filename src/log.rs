#[macro_export]
macro_rules! declmagic_log(
    ($logger:expr, $lvl:expr, $($arg:tt)+) => ({
        static LOC: ::stdlog::LogLocation = ::stdlog::LogLocation {
            line: line!(),
            file: file!(),
            module_path: module_path!(),
        };
        let lvl = $lvl;

        format_args!(|args| {
            $logger.log(&::stdlog::LogRecord {
                level: ::stdlog::LogLevel($lvl),
                args: args,
                file: LOC.file,
                module_path: LOC.module_path,
                line: LOC.line,
            });
        }, $($arg)+)
    })
)

#[macro_export]
macro_rules! declmagic_debug(
    ($logger:expr, $($arg:tt)*) => (if cfg!(not(ndebug)) { declmagic_log!($logger, ::stdlog::DEBUG, $($arg)*) })
)

#[macro_export]
macro_rules! declmagic_error(
    ($logger:expr, $($arg:tt)*) => (declmagic_log!($logger, ::stdlog::ERROR, $($arg)*))
)

#[macro_export]
macro_rules! declmagic_info(
    ($logger:expr, $($arg:tt)*) => (declmagic_log!($logger, ::stdlog::INFO, $($arg)*))
)

#[macro_export]
macro_rules! declmagic_warn(
    ($logger:expr, $($arg:tt)*) => (declmagic_log!($logger, ::stdlog::WARN, $($arg)*))
)

#[deriving(Clone)]
pub struct LogSystem;

impl LogSystem {
    pub fn new() -> LogSystem {
        LogSystem
    }
}

pub trait Logger : ::stdlog::Logger + Clone {
}

impl ::stdlog::Logger for LogSystem {
    fn log(&mut self, record: &::stdlog::LogRecord)
    {
        println!("[{}] at {} ({}:{})\n        {}", record.level, record.module_path, record.file, record.line, record.args);
    }
}

impl Logger for LogSystem {}
