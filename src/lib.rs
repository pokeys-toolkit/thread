#![allow(clippy::uninlined_format_args)]
#![doc(test(attr(ignore)))]

//! # PoKeys Threading Architecture
//!
//! This crate provides a threading architecture for PoKeys devices,
//! allowing each device to operate in its own thread.
//!
//! ## Overview
//!
//! The PoKeys threading architecture is designed to solve the problem of
//! communicating with multiple PoKeys devices from a single application.
//! Each device operates in its own thread, allowing for concurrent
//! communication and state updates without blocking the main application thread.
//!
//! ## Key Components
//!
//! - **ThreadController**: Manages device threads and provides a high-level interface
//!   for device operations.
//! - **DeviceWorker**: Runs in its own thread and handles device communication.
//! - **SharedDeviceState**: Provides thread-safe access to device state.
//! - **StateObserver**: Allows monitoring state changes.
//! - **DeviceOperations**: Provides a high-level interface for device operations.
//! - **DeviceSync**: Handles data synchronization between device and shared state.
//! - **Logger**: Provides configurable logging for threads and controllers.
//!
//! ## Usage Example
//!
//! ```ignore
//! use pokeys_thread::{ThreadControllerBuilder, ThreadController, DeviceOperations};
//! use std::time::Duration;
//!
//! // Create a thread controller
//! let mut controller = ThreadControllerBuilder::new()
//!     .default_refresh_interval(100)
//!     .build();
//!
//! // Discover USB devices
//! let devices = controller.discover_usb_devices().unwrap();
//!
//! if !devices.is_empty() {
//!     // Start a thread for the first device
//!     let thread_id = controller.start_usb_device_thread(devices[0]).unwrap();
//!
//!     // Perform device operations
//!     controller.set_digital_output(thread_id, 1, true).unwrap();
//!
//!     // Wait a bit
//!     std::thread::sleep(Duration::from_millis(100));
//!
//!     // Read input
//!     let input_value = controller.get_digital_input(thread_id, 2).unwrap();
//!     println!("Digital input 2: {}", input_value);
//! }
//! ```
//!
//! ## Features
//!
//! - Thread-safe communication between main thread and device threads
//! - Device state sharing with thread-safe access
//! - Command pattern for thread control
//! - Observer pattern for state change notifications
//! - Comprehensive error handling
//! - Configurable logging system
//! - Support for USB and network devices

pub mod builder;
pub mod commands;
pub mod controller;
pub mod controller_builder;
pub mod error;
pub mod logging;
pub mod observer;
pub mod operations;
pub mod state;
pub mod sync;
pub mod worker;

#[cfg(test)]
mod tests;

// Re-export main types
pub use builder::ThreadWorkerBuilder;
pub use commands::DeviceCommand;
pub use controller::{ThreadController, ThreadControllerImpl};
pub use controller_builder::ThreadControllerBuilder;
pub use error::{Result, ThreadError};
pub use logging::{Logger, SimpleLogger, ThreadLogger};
pub use observer::StateObserver;
pub use operations::DeviceOperations;
pub use state::{DeviceState, SharedDeviceState, StateChangeType, ThreadStatus};
pub use sync::DeviceSync;
pub use worker::{DeviceWorker, DeviceWorkerImpl};
