//! # Thread Controller
//!
//! The thread controller is the main interface for managing device threads.
//! It provides methods for discovering devices, starting threads, and
//! performing device operations.
//!
//! ## Usage Example
//!
//! ```ignore
//! use pokeys_thread::{ThreadControllerBuilder, ThreadController, DeviceOperations};
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
//! }
//! ```

use crate::builder::ThreadWorkerBuilder;
use crate::commands::DeviceCommand;
use crate::error::{Result, ThreadError};
use crate::logging::{Logger, ThreadLogger};
use crate::observer::StateObserver;
use crate::operations::DeviceOperations;
use crate::state::{DeviceState, SharedDeviceState, ThreadStatus};
use pokeys_lib::{ServoConfig, USPIBridgeConfig};
use crate::worker::DeviceWorker;
use log::{debug, error, info, LevelFilter};
use pokeys_lib::{enumerate_network_devices, enumerate_usb_devices, NetworkDeviceSummary};
use std::collections::HashMap;
use std::sync::Arc;

/// Thread controller for managing device threads.
///
/// The thread controller is responsible for:
/// - Discovering devices
/// - Starting and stopping device threads
/// - Sending commands to device threads
/// - Retrieving device state
/// - Creating state observers
/// - Performing device operations
pub trait ThreadController {
    /// Discover USB devices connected to the system.
    ///
    /// # Returns
    ///
    /// A vector of device indices that can be used to connect to the devices.
    ///
    /// # Errors
    ///
    /// Returns an error if the device enumeration fails.
    fn discover_usb_devices(&mut self) -> Result<Vec<u32>>;

    /// Discover network devices on the local network.
    ///
    /// # Parameters
    ///
    /// * `timeout_ms` - The timeout in milliseconds for the discovery process.
    ///
    /// # Returns
    ///
    /// A vector of network device summaries that can be used to connect to the devices.
    ///
    /// # Errors
    ///
    /// Returns an error if the device enumeration fails.
    fn discover_network_devices(&mut self, timeout_ms: u32) -> Result<Vec<NetworkDeviceSummary>>;

    /// Start a thread for a USB device.
    ///
    /// # Parameters
    ///
    /// * `device_index` - The index of the USB device to connect to.
    ///
    /// # Returns
    ///
    /// The thread ID of the newly created thread.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread creation fails or if the device connection fails.
    fn start_usb_device_thread(&mut self, device_index: u32) -> Result<u32>;

    /// Start a thread for a network device.
    ///
    /// # Parameters
    ///
    /// * `device_summary` - The network device summary to connect to.
    ///
    /// # Returns
    ///
    /// The thread ID of the newly created thread.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread creation fails or if the device connection fails.
    fn start_network_device_thread(&mut self, device_summary: NetworkDeviceSummary) -> Result<u32>;

    /// Start a thread for a device with a specific serial number.
    ///
    /// # Parameters
    ///
    /// * `serial_number` - The serial number of the device to connect to.
    /// * `check_network` - Whether to check for network devices.
    /// * `timeout_ms` - The timeout in milliseconds for network device discovery.
    ///
    /// # Returns
    ///
    /// The thread ID of the newly created thread.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread creation fails or if the device connection fails.
    fn start_device_thread_by_serial(
        &mut self,
        serial_number: u32,
        check_network: bool,
        timeout_ms: u32,
    ) -> Result<u32>;

    /// Send a command to a device thread.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `command` - The command to send.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn send_command(&self, thread_id: u32, command: DeviceCommand) -> Result<()>;

    /// Get the status of a device thread.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to get the status of.
    ///
    /// # Returns
    ///
    /// The status of the thread.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found.
    fn get_status(&self, thread_id: u32) -> Result<ThreadStatus>;

    /// Get the state of a device thread.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to get the state of.
    ///
    /// # Returns
    ///
    /// The state of the device.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found.
    fn get_state(&self, thread_id: u32) -> Result<DeviceState>;

    /// Get the shared state of a device thread.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to get the shared state of.
    ///
    /// # Returns
    ///
    /// The shared state of the device.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found.
    fn get_shared_state(&self, thread_id: u32) -> Result<Arc<SharedDeviceState>>;

    /// Create a state observer for a device thread.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to create an observer for.
    ///
    /// # Returns
    ///
    /// A state observer for the thread.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found.
    fn create_observer(&self, thread_id: u32) -> Result<StateObserver>;

    /// Stop all device threads.
    ///
    /// # Errors
    ///
    /// Returns an error if any thread fails to stop.
    fn stop_all(&mut self) -> Result<()>;

    /// Set the log level for a specific thread.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to set the log level for.
    /// * `level` - The log level to set.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the log level set fails.
    fn set_thread_log_level(&mut self, thread_id: u32, level: LevelFilter) -> Result<()>;

    /// Set the log level for all threads and the controller.
    ///
    /// # Parameters
    ///
    /// * `level` - The log level to set.
    ///
    /// # Errors
    ///
    /// Returns an error if any thread fails to set the log level.
    fn set_global_log_level(&mut self, level: LevelFilter) -> Result<()>;

    /// Start model monitoring for a device thread.
    ///
    /// This method starts monitoring the device model file for changes and
    /// updates the device model when changes are detected.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to monitor.
    /// * `model_dir` - Optional custom directory for model files.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if monitoring fails to start.
    fn start_model_monitoring(
        &mut self,
        thread_id: u32,
        model_dir: Option<std::path::PathBuf>,
    ) -> Result<()>;

    /// Stop model monitoring for a device thread.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to stop monitoring.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if monitoring fails to stop.
    fn stop_model_monitoring(&mut self, thread_id: u32) -> Result<()>;

    /// Update the device model for a thread.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to update.
    /// * `model` - The new device model.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the model update fails.
    fn update_device_model(
        &self,
        thread_id: u32,
        model: pokeys_lib::models::DeviceModel,
    ) -> Result<()>;

    /// Check if a thread is currently running.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to check.
    ///
    /// # Returns
    ///
    /// True if the thread is running, false otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found.
    fn is_thread_running(&self, thread_id: u32) -> Result<bool> {
        let status = self.get_status(thread_id)?;
        Ok(status == ThreadStatus::Running)
    }

    /// Get a list of all active thread IDs.
    ///
    /// # Returns
    ///
    /// A vector of thread IDs that are currently active.
    fn list_active_threads(&self) -> Result<Vec<u32>>;

    /// Stop a specific thread.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to stop.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or fails to stop.
    fn stop_thread(&mut self, thread_id: u32) -> Result<()>;
}

/// Thread controller implementation.
///
/// This is the main implementation of the `ThreadController` trait.
/// It manages device threads and provides methods for device operations.
/// Thread controller implementation.
///
/// This is the main implementation of the `ThreadController` trait.
/// It manages device threads and provides methods for device operations.
pub struct ThreadControllerImpl {
    /// Device threads
    threads: HashMap<u32, Box<dyn DeviceWorker>>,
    /// Next thread ID
    next_thread_id: u32,
    /// Default refresh interval in milliseconds
    default_refresh_interval: u64,
    /// Logger
    logger: Option<Arc<dyn Logger>>,
    /// Model monitors
    model_monitors: HashMap<u32, pokeys_lib::models::ModelMonitor>,
}

impl Default for ThreadControllerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreadControllerImpl {
    /// Create a new thread controller.
    ///
    /// # Returns
    ///
    /// A new thread controller with default settings.
    pub fn new() -> Self {
        Self {
            threads: HashMap::new(),
            next_thread_id: 1,
            default_refresh_interval: 100, // Default refresh interval: 100ms
            logger: None,
            model_monitors: HashMap::new(),
        }
    }

    /// Create a new thread controller with a logger.
    ///
    /// # Parameters
    ///
    /// * `logger` - The logger to use.
    ///
    /// # Returns
    ///
    /// A new thread controller with the specified logger.
    pub fn with_logger(logger: Arc<dyn Logger>) -> Self {
        Self {
            threads: HashMap::new(),
            next_thread_id: 1,
            default_refresh_interval: 100,
            logger: Some(logger),
            model_monitors: HashMap::new(),
        }
    }

    /// Set the default refresh interval.
    ///
    /// # Parameters
    ///
    /// * `interval_ms` - The refresh interval in milliseconds.
    pub fn set_default_refresh_interval(&mut self, interval_ms: u64) {
        self.default_refresh_interval = interval_ms;
    }

    /// Set the logger.
    ///
    /// # Parameters
    ///
    /// * `logger` - The logger to use.
    pub fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.logger = Some(logger);
    }

    /// Get the next thread ID.
    ///
    /// # Returns
    ///
    /// The next available thread ID.
    fn next_thread_id(&mut self) -> u32 {
        let id = self.next_thread_id;
        self.next_thread_id += 1;
        id
    }

    /// Get a device thread by ID.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to get.
    ///
    /// # Returns
    ///
    /// A reference to the thread.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found.
    fn get_thread(&self, thread_id: u32) -> Result<&dyn DeviceWorker> {
        self.threads
            .get(&thread_id)
            .ok_or(ThreadError::ThreadNotFound(thread_id))
            .map(|boxed| boxed.as_ref())
    }

    /// Get a mutable device thread by ID.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to get.
    ///
    /// # Returns
    ///
    /// A mutable reference to the thread.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found.
    fn get_thread_mut(&mut self, thread_id: u32) -> Result<&mut Box<dyn DeviceWorker>> {
        self.threads
            .get_mut(&thread_id)
            .ok_or(ThreadError::ThreadNotFound(thread_id))
    }

    /// Log a message.
    ///
    /// # Parameters
    ///
    /// * `level` - The log level.
    /// * `message` - The message to log.
    fn log(&self, level: log::Level, message: &str) {
        if let Some(logger) = &self.logger {
            logger.log(level, "ThreadController", message);
        } else {
            match level {
                log::Level::Error => error!("{message}"),
                log::Level::Warn => log::warn!("{message}"),
                log::Level::Info => info!("{message}"),
                log::Level::Debug => debug!("{message}"),
                log::Level::Trace => log::trace!("{message}"),
            }
        }
    }
}

impl ThreadController for ThreadControllerImpl {
    fn discover_usb_devices(&mut self) -> Result<Vec<u32>> {
        self.log(log::Level::Info, "Discovering USB devices");

        let device_count = enumerate_usb_devices().map_err(ThreadError::DeviceError)?;

        let mut device_indices = Vec::new();
        for i in 0..device_count {
            device_indices.push(i as u32);
        }

        Ok(device_indices)
    }

    fn discover_network_devices(&mut self, timeout_ms: u32) -> Result<Vec<NetworkDeviceSummary>> {
        self.log(
            log::Level::Info,
            &format!("Discovering network devices with timeout {timeout_ms}ms"),
        );

        let devices = enumerate_network_devices(timeout_ms).map_err(ThreadError::DeviceError)?;

        Ok(devices)
    }

    fn start_usb_device_thread(&mut self, device_index: u32) -> Result<u32> {
        self.log(
            log::Level::Info,
            &format!("Starting USB device thread for device index {device_index}"),
        );

        // Generate a new thread ID
        let thread_id = self.next_thread_id();

        // Create a device worker
        let mut builder =
            ThreadWorkerBuilder::new(thread_id).refresh_interval(self.default_refresh_interval);

        // Add logger if available
        if let Some(logger) = &self.logger {
            let thread_logger = Arc::new(ThreadLogger::new(thread_id, logger.clone()));
            builder = builder.with_logger(thread_logger);
        }

        let worker = builder.build_usb_device(device_index)?;

        // Store the worker
        self.threads.insert(thread_id, worker);

        // Automatically start model monitoring
        if let Err(e) = self.start_model_monitoring(thread_id, None) {
            self.log(
                log::Level::Warn,
                &format!("Failed to start model monitoring for thread {thread_id}: {e}"),
            );
            // Continue even if model monitoring fails
        }

        Ok(thread_id)
    }

    fn start_network_device_thread(&mut self, device_summary: NetworkDeviceSummary) -> Result<u32> {
        self.log(
            log::Level::Info,
            &format!(
                "Starting network device thread for device with serial {}",
                device_summary.serial_number
            ),
        );

        // Generate a new thread ID
        let thread_id = self.next_thread_id();

        // Create a device worker
        let mut builder =
            ThreadWorkerBuilder::new(thread_id).refresh_interval(self.default_refresh_interval);

        // Add logger if available
        if let Some(logger) = &self.logger {
            let thread_logger = Arc::new(ThreadLogger::new(thread_id, logger.clone()));
            builder = builder.with_logger(thread_logger);
        }

        let worker = builder.build_network_device(device_summary)?;

        // Store the worker
        self.threads.insert(thread_id, worker);

        // Automatically start model monitoring
        if let Err(e) = self.start_model_monitoring(thread_id, None) {
            self.log(
                log::Level::Warn,
                &format!("Failed to start model monitoring for thread {thread_id}: {e}"),
            );
            // Continue even if model monitoring fails
        }

        Ok(thread_id)
    }

    fn start_device_thread_by_serial(
        &mut self,
        serial_number: u32,
        check_network: bool,
        timeout_ms: u32,
    ) -> Result<u32> {
        self.log(
            log::Level::Info,
            &format!("Starting device thread for device with serial {serial_number}"),
        );

        // Generate a new thread ID
        let thread_id = self.next_thread_id();

        // Create a device worker
        let mut builder =
            ThreadWorkerBuilder::new(thread_id).refresh_interval(self.default_refresh_interval);

        // Add logger if available
        if let Some(logger) = &self.logger {
            let thread_logger = Arc::new(ThreadLogger::new(thread_id, logger.clone()));
            builder = builder.with_logger(thread_logger);
        }

        let worker = builder.build_device_by_serial(serial_number, check_network, timeout_ms)?;

        // Store the worker
        self.threads.insert(thread_id, worker);

        // Automatically start model monitoring
        if let Err(e) = self.start_model_monitoring(thread_id, None) {
            self.log(
                log::Level::Warn,
                &format!("Failed to start model monitoring for thread {thread_id}: {e}"),
            );
            // Continue even if model monitoring fails
        }

        Ok(thread_id)
    }

    fn send_command(&self, thread_id: u32, command: DeviceCommand) -> Result<()> {
        self.log(
            log::Level::Debug,
            &format!("Sending command {command:?} to thread {thread_id}"),
        );

        let thread = self.get_thread(thread_id)?;
        thread.send_command(command)
    }

    fn get_status(&self, thread_id: u32) -> Result<ThreadStatus> {
        let thread = self.get_thread(thread_id)?;
        Ok(thread.status())
    }

    fn get_state(&self, thread_id: u32) -> Result<DeviceState> {
        let thread = self.get_thread(thread_id)?;
        let shared_state = thread.shared_state();

        Ok(shared_state.read(|state| state.clone()))
    }

    fn get_shared_state(&self, thread_id: u32) -> Result<Arc<SharedDeviceState>> {
        let thread = self.get_thread(thread_id)?;
        Ok(thread.shared_state())
    }

    fn create_observer(&self, thread_id: u32) -> Result<StateObserver> {
        let thread = self.get_thread(thread_id)?;
        let shared_state = thread.shared_state();
        Ok(StateObserver::new(thread_id, shared_state))
    }

    fn stop_all(&mut self) -> Result<()> {
        self.log(log::Level::Info, "Stopping all device threads");

        let mut errors = Vec::new();

        // Send terminate command to all threads
        for (thread_id, thread) in &self.threads {
            if let Err(e) = thread.send_command(DeviceCommand::Terminate) {
                let error_msg = format!("Failed to terminate thread {thread_id}: {e}");
                self.log(log::Level::Error, &error_msg);
                errors.push(e);
            }
        }

        // Wait for all threads to finish
        // In a real implementation, we would join all threads here

        // Clear the threads map
        self.threads.clear();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ThreadError::ThreadJoinError)
        }
    }

    fn set_thread_log_level(&mut self, thread_id: u32, level: LevelFilter) -> Result<()> {
        self.log(
            log::Level::Info,
            &format!("Setting log level for thread {thread_id} to {level:?}"),
        );

        let thread = self.get_thread_mut(thread_id)?;
        thread.set_log_level(level)
    }

    fn set_global_log_level(&mut self, level: LevelFilter) -> Result<()> {
        self.log(
            log::Level::Info,
            &format!("Setting global log level to {level:?}"),
        );

        // Set log level for the controller logger
        if let Some(ref mut logger) = self.logger {
            let mut_logger: &mut dyn Logger = Arc::get_mut(logger).ok_or_else(|| {
                ThreadError::LockPoisoned("Failed to get mutable reference to logger".to_string())
            })?;
            mut_logger.set_level(level);
        }

        // Set log level for all threads
        let mut errors = Vec::new();
        let thread_ids: Vec<u32> = self.threads.keys().cloned().collect();

        for thread_id in thread_ids {
            if let Err(e) = self.set_thread_log_level(thread_id, level) {
                let error_msg = format!("Failed to set log level for thread {thread_id}: {e}");
                self.log(log::Level::Error, &error_msg);
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ThreadError::OperationFailed(format!(
                "Failed to set log level for {} threads",
                errors.len()
            )))
        }
    }

    fn start_model_monitoring(
        &mut self,
        thread_id: u32,
        model_dir: Option<std::path::PathBuf>,
    ) -> Result<()> {
        self.log(
            log::Level::Info,
            &format!("Starting model monitoring for thread {thread_id}"),
        );

        // Check if the thread exists
        let thread = self.get_thread(thread_id)?;

        // Get the device state
        let state = thread.shared_state().read(|state| state.clone());

        // Get the device model name based on device type
        match state.device_data.device_type_id {
            10 => {} // DeviceTypeId::Device56U
            30 => {} // DeviceTypeId::Device57U
            31 => {} // DeviceTypeId::Device57E
            11 => {} // DeviceTypeId::Device56E
            _ => return Err(ThreadError::UnsupportedDevice),
        };

        // Create a model monitor
        let dir = model_dir.unwrap_or_else(|| {
            let mut path = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            path.push(".config/pokeys/models");
            path
        });

        // Create the directory if it doesn't exist
        if !dir.exists() {
            std::fs::create_dir_all(&dir).map_err(|e| {
                ThreadError::Other(format!("Failed to create model directory: {}", e))
            })?;
        }

        // Copy default models to the user's directory
        if let Err(e) = pokeys_lib::models::copy_default_models_to_user_dir(Some(&dir)) {
            self.log(
                log::Level::Warn,
                &format!("Failed to copy default models: {}", e),
            );
            // Continue even if copying fails
        }

        // Create a thread-safe command sender
        let (tx, rx) = crossbeam_channel::unbounded::<DeviceCommand>();

        // Create a callback that sends model updates to the device thread
        let tx_clone = tx.clone();
        let callback = move |_: String, model: pokeys_lib::models::DeviceModel| {
            // Send the model update command through the channel
            let _ = tx_clone.send(DeviceCommand::UpdateModel(model.clone()));
        };

        // Create and start the model monitor
        let mut monitor = pokeys_lib::models::ModelMonitor::new(dir, callback);
        monitor
            .start()
            .map_err(|e| ThreadError::Other(format!("Failed to start model monitoring: {}", e)))?;

        // Store the monitor
        self.model_monitors.insert(thread_id, monitor);

        // Create a thread to forward commands from the channel to the device thread
        let thread_sender = self.get_thread(thread_id)?.command_sender().clone();
        std::thread::spawn(move || {
            while let Ok(command) = rx.recv() {
                // Send the command directly to the thread
                let _ = thread_sender.send(command);
            }
        });

        Ok(())
    }

    fn stop_model_monitoring(&mut self, thread_id: u32) -> Result<()> {
        self.log(
            log::Level::Info,
            &format!("Stopping model monitoring for thread {thread_id}"),
        );

        // Check if the thread exists
        if !self.threads.contains_key(&thread_id) {
            return Err(ThreadError::ThreadNotFound(thread_id));
        }

        // Stop and remove the monitor
        if let Some(mut monitor) = self.model_monitors.remove(&thread_id) {
            monitor.stop().map_err(|e| {
                ThreadError::Other(format!("Failed to stop model monitoring: {}", e))
            })?;
        }

        Ok(())
    }

    fn update_device_model(
        &self,
        thread_id: u32,
        model: pokeys_lib::models::DeviceModel,
    ) -> Result<()> {
        self.log(
            log::Level::Info,
            &format!(
                "Updating device model for thread {thread_id} to {}",
                model.name
            ),
        );

        // Send the model update command to the device thread
        self.send_command(thread_id, DeviceCommand::UpdateModel(model))
    }

    fn list_active_threads(&self) -> Result<Vec<u32>> {
        let thread_ids: Vec<u32> = self.threads.keys().copied().collect();
        Ok(thread_ids)
    }

    fn stop_thread(&mut self, thread_id: u32) -> Result<()> {
        self.log(log::Level::Info, &format!("Stopping thread {thread_id}"));

        if let Some(mut worker) = self.threads.remove(&thread_id) {
            worker.stop()?;
            self.log(
                log::Level::Info,
                &format!("Thread {thread_id} stopped successfully"),
            );
            Ok(())
        } else {
            Err(ThreadError::ThreadNotFound(thread_id))
        }
    }
}

impl DeviceOperations for ThreadControllerImpl {
    fn set_digital_output(&self, thread_id: u32, pin: u32, value: bool) -> Result<()> {
        self.log(
            log::Level::Debug,
            &format!("Setting digital output pin {pin} to {value} on thread {thread_id}"),
        );
        self.send_command(thread_id, DeviceCommand::SetDigitalOutput { pin, value })
    }

    fn get_digital_input(&self, thread_id: u32, pin: u32) -> Result<bool> {
        self.log(
            log::Level::Debug,
            &format!("Getting digital input pin {pin} from thread {thread_id}"),
        );
        let shared_state = self.get_shared_state(thread_id)?;
        shared_state
            .get_digital_input(pin)
            .ok_or_else(|| ThreadError::InvalidParameter(format!("Invalid pin: {pin}")))
    }

    fn set_analog_output(&self, thread_id: u32, pin: u32, value: u32) -> Result<()> {
        self.log(
            log::Level::Debug,
            &format!("Setting analog output pin {pin} to {value} on thread {thread_id}"),
        );
        self.send_command(thread_id, DeviceCommand::SetAnalogOutput { pin, value })
    }

    fn get_analog_input(&self, thread_id: u32, pin: u32) -> Result<u32> {
        self.log(
            log::Level::Debug,
            &format!("Getting analog input pin {pin} from thread {thread_id}"),
        );
        let shared_state = self.get_shared_state(thread_id)?;
        shared_state
            .get_analog_input(pin)
            .ok_or_else(|| ThreadError::InvalidParameter(format!("Invalid pin: {pin}")))
    }

    fn set_pwm_duty_cycle(&self, thread_id: u32, channel: usize, duty: u32) -> Result<()> {
        self.log(
            log::Level::Debug,
            &format!("Setting PWM channel {channel} duty to {duty} on thread {thread_id}"),
        );
        self.send_command(thread_id, DeviceCommand::SetPwmDuty { channel, duty })
    }

    fn set_pwm_duty_cycle_percent(
        &self,
        thread_id: u32,
        channel: usize,
        duty_percent: f32,
    ) -> Result<()> {
        self.log(
            log::Level::Debug,
            &format!("Setting PWM channel {channel} duty to {duty_percent}% on thread {thread_id}"),
        );

        // Convert percentage to raw duty cycle value (0-4095 for 12-bit)
        let duty = ((duty_percent / 100.0) * 4095.0) as u32;
        self.set_pwm_duty_cycle(thread_id, channel, duty)
    }

    fn configure_servo(&self, thread_id: u32, pin: u8, servo_config: ServoConfig) -> Result<()> {
        self.log(
            log::Level::Debug,
            &format!("Configuring servo on pin {pin} for thread {thread_id}"),
        );
        self.send_command(thread_id, DeviceCommand::ConfigureServo { pin, config: servo_config })
    }

    fn set_servo_angle(&self, thread_id: u32, pin: u8, angle: f32) -> Result<()> {
        self.log(
            log::Level::Debug,
            &format!("Setting servo angle on pin {pin} to {angle}° for thread {thread_id}"),
        );
        self.send_command(thread_id, DeviceCommand::SetServoAngle { pin, angle })
    }

    fn set_servo_speed(&self, thread_id: u32, pin: u8, speed: f32) -> Result<()> {
        self.log(
            log::Level::Debug,
            &format!("Setting servo speed on pin {pin} to {speed} for thread {thread_id}"),
        );
        self.send_command(thread_id, DeviceCommand::SetServoSpeed { pin, speed })
    }

    fn stop_servo(&self, thread_id: u32, pin: u8) -> Result<()> {
        self.log(
            log::Level::Debug,
            &format!("Stopping servo on pin {pin} for thread {thread_id}"),
        );
        self.send_command(thread_id, DeviceCommand::StopServo { pin })
    }

    fn i2c_write(&self, thread_id: u32, address: u8, data: Vec<u8>) -> Result<()> {
        self.log(
            log::Level::Debug,
            &format!("I2C write to address 0x{:02X} on thread {}", address, thread_id),
        );
        self.send_command(thread_id, DeviceCommand::I2cWrite { address, data })
    }

    fn i2c_read(&self, thread_id: u32, address: u8, length: u8) -> Result<Vec<u8>> {
        self.log(
            log::Level::Debug,
            &format!("I2C read from address 0x{:02X} on thread {}", address, thread_id),
        );
        // For now, return empty vector - full implementation would need response channel
        self.send_command(thread_id, DeviceCommand::I2cRead { address, length })?;
        Ok(Vec::new())
    }

    fn i2c_write_read(&self, thread_id: u32, address: u8, write_data: Vec<u8>, read_length: u8) -> Result<Vec<u8>> {
        self.log(
            log::Level::Debug,
            &format!("I2C write-read to address 0x{:02X} on thread {}", address, thread_id),
        );
        // For now, return empty vector - full implementation would need response channel
        self.send_command(thread_id, DeviceCommand::I2cWriteRead { address, write_data, read_length })?;
        Ok(Vec::new())
    }

    fn i2c_scan(&self, thread_id: u32) -> Result<Vec<u8>> {
        self.log(
            log::Level::Debug,
            &format!("I2C scan on thread {}", thread_id),
        );
        // For now, return empty vector - full implementation would need response channel
        self.send_command(thread_id, DeviceCommand::I2cScan)?;
        Ok(Vec::new())
    }

    fn configure_uspibridge(&self, thread_id: u32, config: USPIBridgeConfig) -> Result<()> {
        self.log(
            log::Level::Debug,
            &format!("Configuring uSPIBridge on thread {}", thread_id),
        );
        self.send_command(thread_id, DeviceCommand::ConfigureUSPIBridge { config })
    }

    fn uspibridge_command(&self, thread_id: u32, command: Vec<u8>) -> Result<Vec<u8>> {
        self.log(
            log::Level::Debug,
            &format!("Sending uSPIBridge command on thread {}", thread_id),
        );
        // For now, return empty vector - full implementation would need response channel
        self.send_command(thread_id, DeviceCommand::USPIBridgeCommand { command })?;
        Ok(Vec::new())
    }

    fn get_encoder_value(&self, thread_id: u32, encoder_index: u32) -> Result<i32> {
        self.log(
            log::Level::Debug,
            &format!("Getting encoder {encoder_index} value from thread {thread_id}"),
        );
        let shared_state = self.get_shared_state(thread_id)?;
        shared_state
            .get_encoder_value(encoder_index)
            .ok_or_else(|| {
                ThreadError::InvalidParameter(format!("Invalid encoder index: {encoder_index}"))
            })
    }

    fn configure_encoder(
        &self,
        thread_id: u32,
        encoder_index: u32,
        pin_a: u32,
        pin_b: u32,
        enabled: bool,
        sampling_4x: bool,
    ) -> Result<()> {
        self.log(
            log::Level::Debug,
            &format!(
                "Configuring encoder {encoder_index} on pins {pin_a} and {pin_b} (enabled: {enabled}, 4x: {sampling_4x}) on thread {thread_id}"
            ),
        );

        self.send_command(
            thread_id,
            DeviceCommand::ConfigureEncoder {
                encoder_index,
                pin_a,
                pin_b,
                enabled,
                sampling_4x,
            },
        )
    }

    fn reset_digital_counter(&self, thread_id: u32, pin: u32) -> Result<()> {
        self.log(
            log::Level::Debug,
            &format!("Resetting digital counter for pin {pin} on thread {thread_id}"),
        );
        self.send_command(thread_id, DeviceCommand::ResetDigitalCounter { pin })
    }

    fn send_custom_request(
        &self,
        thread_id: u32,
        request_type: u8,
        param1: u8,
        param2: u8,
        param3: u8,
        param4: u8,
    ) -> Result<()> {
        self.log(
            log::Level::Debug,
            &format!(
                "Sending custom request: {request_type:02X} {param1:02X} {param2:02X} {param3:02X} {param4:02X} to thread {thread_id}"
            ),
        );

        self.send_command(
            thread_id,
            DeviceCommand::Custom {
                request_type,
                param1,
                param2,
                param3,
                param4,
            },
        )
    }

    fn set_pin_function(
        &self,
        thread_id: u32,
        pin: u32,
        pin_function: pokeys_lib::PinFunction,
    ) -> Result<()> {
        self.log(
            log::Level::Debug,
            &format!(
                "Setting pin {pin} function to {:?} on thread {thread_id}",
                pin_function
            ),
        );
        self.send_command(
            thread_id,
            DeviceCommand::SetPinFunction { pin, pin_function },
        )
    }
}

impl Drop for ThreadControllerImpl {
    fn drop(&mut self) {
        // Try to stop all threads when the controller is dropped
        if let Err(e) = self.stop_all() {
            self.log(
                log::Level::Error,
                &format!("Failed to stop all threads during controller drop: {e}"),
            );
        }
    }
}
