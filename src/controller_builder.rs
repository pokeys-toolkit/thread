//! Builder for creating thread controllers

use crate::controller::ThreadControllerImpl;
use crate::logging::Logger;
use std::path::PathBuf;
use std::sync::Arc;

/// Builder for creating thread controllers
pub struct ThreadControllerBuilder {
    /// Default refresh interval in milliseconds
    default_refresh_interval: u64,
    /// Logger
    logger: Option<Arc<dyn Logger>>,
    /// Model directory
    model_dir: Option<PathBuf>,
}

impl ThreadControllerBuilder {
    /// Create a new thread controller builder
    pub fn new() -> Self {
        Self {
            default_refresh_interval: 100, // Default refresh interval: 100ms
            logger: None,
            model_dir: None,
        }
    }

    /// Set the default refresh interval
    pub fn default_refresh_interval(mut self, interval_ms: u64) -> Self {
        self.default_refresh_interval = interval_ms;
        self
    }

    /// Set the logger
    pub fn with_logger(mut self, logger: Arc<dyn Logger>) -> Self {
        self.logger = Some(logger);
        self
    }

    /// Set the model directory
    pub fn model_dir(mut self, dir: Option<PathBuf>) -> Self {
        self.model_dir = dir;
        self
    }

    /// Build a thread controller
    pub fn build(self) -> ThreadControllerImpl {
        let mut controller = if let Some(logger) = self.logger {
            ThreadControllerImpl::with_logger(logger)
        } else {
            ThreadControllerImpl::new()
        };

        controller.set_default_refresh_interval(self.default_refresh_interval);
        controller
    }
}

impl Default for ThreadControllerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
