//! Logging utilities

use log::{debug, error, info, warn, Level, LevelFilter};
use std::sync::Arc;

/// Logger trait for configurable logging
pub trait Logger: Send + Sync {
    /// Log a message
    fn log(&self, level: Level, target: &str, message: &str);

    /// Set the log level
    fn set_level(&mut self, level: LevelFilter);

    /// Get the log level
    fn level(&self) -> LevelFilter;
}

/// Simple logger implementation
pub struct SimpleLogger {
    /// Log level
    level: LevelFilter,
}

impl SimpleLogger {
    /// Create a new simple logger
    pub fn new(level: LevelFilter) -> Self {
        Self { level }
    }
}

impl Logger for SimpleLogger {
    fn log(&self, level: Level, target: &str, message: &str) {
        match level {
            Level::Error => error!(target: target, "{message}"),
            Level::Warn => warn!(target: target, "{message}"),
            Level::Info => info!(target: target, "{message}"),
            Level::Debug => debug!(target: target, "{message}"),
            Level::Trace => log::trace!(target: target, "{message}"),
        }
    }

    fn set_level(&mut self, level: LevelFilter) {
        self.level = level;
    }

    fn level(&self) -> LevelFilter {
        self.level
    }
}

/// Thread-specific logger
pub struct ThreadLogger {
    /// Inner logger
    inner: Arc<dyn Logger>,
    /// Thread ID
    thread_id: u32,
}

impl ThreadLogger {
    /// Create a new thread logger
    pub fn new(thread_id: u32, inner: Arc<dyn Logger>) -> Self {
        Self { inner, thread_id }
    }

    /// Log an error message
    pub fn error(&self, message: &str) {
        self.inner
            .log(Level::Error, &format!("Thread {}", self.thread_id), message);
    }

    /// Log a warning message
    pub fn warn(&self, message: &str) {
        self.inner
            .log(Level::Warn, &format!("Thread {}", self.thread_id), message);
    }

    /// Log an info message
    pub fn info(&self, message: &str) {
        self.inner
            .log(Level::Info, &format!("Thread {}", self.thread_id), message);
    }

    /// Log a debug message
    pub fn debug(&self, message: &str) {
        self.inner
            .log(Level::Debug, &format!("Thread {}", self.thread_id), message);
    }

    /// Log a trace message
    pub fn trace(&self, message: &str) {
        self.inner
            .log(Level::Trace, &format!("Thread {}", self.thread_id), message);
    }
}
