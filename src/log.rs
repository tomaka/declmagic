/// Entry in the logs.
pub struct LogRecord {
    level: LogLevel,
    message: String
}

/// Level of a log entry.
pub enum LogLevel {
    Warning,
    Error
}

impl LogRecord {
    pub fn new(level: LogLevel, message: String) -> LogRecord {
        LogRecord {
            level: level,
            message: message
        }
    }
}
