use colored::*;

#[derive(Debug, Clone, Copy)]
pub enum Level {
    Error = 0,
    Warn,
    Info,
    Debug,
    Trace,
}

impl<T: AsRef<str>> From<T> for Level {
    fn from(str: T) -> Self {
        match str.as_ref() {
            "ERROR" => Self::Error,
            "WARN" => Self::Warn,
            "INFO" => Self::Info,
            "DEBUG" => Self::Debug,
            _ => Self::Trace,
        }
    }
}

pub struct Logger {
    level: Level,
}

impl Logger {
    pub fn new(level: Level) -> Self {
        Self { level }
    }

    pub fn log(&self, depth: usize, level: Level, msg: String) {
        if (level as u32) <= (self.level as u32) {
            let (l, m) = match level {
                Level::Error => ("ERROR".red(), msg.red()),
                Level::Warn => ("WARN".yellow(), msg.yellow()),
                Level::Info => ("INFO".green(), msg.green()),
                Level::Debug => ("DEBUG".cyan(), msg.cyan()),
                Level::Trace => ("TRACE".normal(), msg.normal()),
            };

            println!("{}[{:5}] {}", "  ".repeat(depth), l, m);
        }
    }
}
