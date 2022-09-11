// The logging format for OSConfig is as follows:
// [<timestamp>] [<short file>:<line number>] [[ERROR]] <log message>

// TODO: separate parsing and formatting of log messages

// TODO: use the standard "log" crate for a basic logging interface

use chrono::NaiveDateTime;
use colored::Colorize;
use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct Log {
    pub timestamp: NaiveDateTime,
    pub file: String,
    pub line: u32,
    pub level: Level,
    pub trace: String,
}

#[derive(Debug, PartialEq)]
pub enum Level {
    Info,
    Error
}

// impl From<&str> for Trace {
//     fn from(s: &str) -> Self {
//         // TODO: might need to trim the string
//         if s.starts_with("[ERROR]") {
//             Trace::Error(s.to_string())
//         } else {
//             Trace::Info(s.to_string())
//         }
//     }
// }

impl Log {
    pub fn trace(s: String) -> Result<Self, Box<dyn Error>> {
        // TODO: lazy static here
        let re = regex::Regex::new(
            r"\[(([0-9]{4})-([0-1][0-9])-([0-3][0-9])\s([0-1][0-9]|[2][0-3]):([0-5][0-9]):([0-5][0-9]))\]\s\[(.*):(\d*)\]\s((\[ERROR\]\s)?(.*))",
        )?;

        let caps = re.captures(&s).ok_or("Invalid log format")?;
        let timestamp = NaiveDateTime::parse_from_str(&caps[1], "%Y-%m-%d %H:%M:%S")?;
        let file = caps[8].to_string();
        let line = caps[9].parse::<u32>()?;
        //
        let level = if caps[10].starts_with("[ERROR]") {
            Level::Error
        } else {
            Level::Info
        };
        let trace = caps[12].to_string();
        Ok(Log {
            timestamp,
            file,
            line,
            level,
            trace,
        })
    }
}

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Colorize output of logs
        write!(
            f,
            "[{}] [{}:{}] {}{}",
            self.timestamp.to_string().bright_blue(),
            self.file.bright_green(),
            self.line.to_string().bright_yellow(),
            match self.level {
                Level::Info => "".bright_black(),
                Level::Error => "[ERROR] ".bright_red(),
            },
            self.trace
        )
    }
}