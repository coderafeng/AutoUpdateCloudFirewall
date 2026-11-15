//! In-memory + on-disk activity log.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use chrono::Local;
use serde::Serialize;

use crate::storage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Level {
    Info,
    Success,
    Warn,
    Error,
}

impl Level {
    pub fn label(self) -> &'static str {
        match self {
            Level::Info => "INFO",
            Level::Success => "OK",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }

    fn parse(text: &str) -> Level {
        match text.trim() {
            "OK" | "SUCCESS" => Level::Success,
            "WARN" | "WARNING" => Level::Warn,
            "ERROR" => Level::Error,
            _ => Level::Info,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub at: String,
    pub level: Level,
    pub message: String,
}

impl LogEntry {
    pub fn line(&self) -> String {
        format!("{} [{}] {}", self.at, self.level.label(), self.message)
    }

    fn parse(line: &str) -> Option<LogEntry> {
        let line = line.trim_end();
        if line.is_empty() {
            return None;
        }
        // 2026-11-14 12:00:01 [INFO] message
        let (at, rest) = line.split_once(" [")?;
        let (level, message) = rest.split_once("] ")?;
        Some(LogEntry {
            at: at.to_string(),
            level: Level::parse(level),
            message: message.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Logger {
    path: PathBuf,
}

impl Logger {
    pub fn new() -> Self {
        let path = storage::log_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        Self { path }
    }

    /// Append a line to the log file and return the entry for the UI.
    pub fn log(&self, level: Level, message: impl Into<String>) -> LogEntry {
        let entry = LogEntry {
            at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            level,
            message: message.into(),
        };
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = writeln!(file, "{}", entry.line());
        }
        entry
    }

    pub fn read_recent(&self, limit: usize) -> Vec<LogEntry> {
        let Ok(text) = std::fs::read_to_string(&self.path) else {
            return Vec::new();
        };
        let mut entries: Vec<LogEntry> = text.lines().filter_map(LogEntry::parse).collect();
        if entries.len() > limit {
            entries = entries.split_off(entries.len() - limit);
        }
        entries
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_log_line() {
        let entry = LogEntry::parse("2026-11-14 10:00:00 [WARN] something odd").unwrap();
        assert_eq!(entry.at, "2026-11-14 10:00:00");
        assert_eq!(entry.level, Level::Warn);
        assert_eq!(entry.message, "something odd");
    }
}
