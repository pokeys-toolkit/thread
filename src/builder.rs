//! Builder for creating device threads

use crate::error::{Result, ThreadError};
use crate::logging::ThreadLogger;
use crate::worker::{DeviceWorker, DeviceWorkerImpl};
use log::info;
use pokeys_lib::{
    connect_to_device, connect_to_device_with_serial, connect_to_network_device,
    NetworkDeviceSummary,
};
use std::sync::Arc;

/// Builder for creating device threads
pub struct ThreadWorkerBuilder {
    /// Thread ID
    thread_id: u32,
    /// Refresh interval in milliseconds
    refresh_interval: u64,
    /// Logger
    logger: Option<Arc<ThreadLogger>>,
}

impl ThreadWorkerBuilder {
    /// Create a new thread worker builder
    pub fn new(thread_id: u32) -> Self {
        Self {
            thread_id,
            refresh_interval: 100, // Default refresh interval: 100ms
            logger: None,
        }
    }

    /// Set the refresh interval
    pub fn refresh_interval(mut self, interval_ms: u64) -> Self {
        self.refresh_interval = interval_ms;
        self
    }

    /// Set the logger
    pub fn with_logger(mut self, logger: Arc<ThreadLogger>) -> Self {
        self.logger = Some(logger);
        self
    }

    /// Build a device worker for a USB device
    pub fn build_usb_device(self, device_index: u32) -> Result<Box<dyn DeviceWorker>> {
        if let Some(logger) = &self.logger {
            logger.info(&format!(
                "Creating USB device thread for device index {}",
                device_index
            ));
        } else {
            info!(
                "Creating USB device thread for device index {}",
                device_index
            );
        }

        // Connect to the device to get initial state
        let device = connect_to_device(device_index).map_err(ThreadError::DeviceError)?;

        // Create the device worker
        let (mut worker, _command_rx) = DeviceWorkerImpl::new_usb(
            self.thread_id,
            &device,
            device_index,
            self.refresh_interval,
        )?;

        // Add logger if available
        if let Some(logger) = self.logger {
            worker = worker.with_logger(logger);
        }

        // Create a boxed worker
        let mut boxed_worker: Box<dyn DeviceWorker> = Box::new(worker);

        // Start the worker thread
        boxed_worker.start()?;

        Ok(boxed_worker)
    }

    /// Build a device worker for a network device
    pub fn build_network_device(
        self,
        device_summary: NetworkDeviceSummary,
    ) -> Result<Box<dyn DeviceWorker>> {
        if let Some(logger) = &self.logger {
            logger.info(&format!(
                "Creating network device thread for device with serial {}",
                device_summary.serial_number
            ));
        } else {
            info!(
                "Creating network device thread for device with serial {}",
                device_summary.serial_number
            );
        }

        // Connect to the device to get initial state
        let device =
            connect_to_network_device(&device_summary).map_err(ThreadError::DeviceError)?;

        // Create the device worker
        let (mut worker, _command_rx) = DeviceWorkerImpl::new_network(
            self.thread_id,
            &device,
            device_summary,
            self.refresh_interval,
        )?;

        // Add logger if available
        if let Some(logger) = self.logger {
            worker = worker.with_logger(logger);
        }

        // Create a boxed worker
        let mut boxed_worker: Box<dyn DeviceWorker> = Box::new(worker);

        // Start the worker thread
        boxed_worker.start()?;

        Ok(boxed_worker)
    }

    /// Build a device worker for a device with a specific serial number
    pub fn build_device_by_serial(
        self,
        serial_number: u32,
        check_network: bool,
        timeout_ms: u32,
    ) -> Result<Box<dyn DeviceWorker>> {
        if let Some(logger) = &self.logger {
            logger.info(&format!(
                "Creating device thread for device with serial {}",
                serial_number
            ));
        } else {
            info!(
                "Creating device thread for device with serial {}",
                serial_number
            );
        }

        // Connect to the device to get initial state
        let device = connect_to_device_with_serial(serial_number, check_network, timeout_ms)
            .map_err(ThreadError::DeviceError)?;

        // Determine device type based on connection type
        let (mut worker, _command_rx) = match device.get_connection_type() {
            pokeys_lib::DeviceConnectionType::UsbDevice
            | pokeys_lib::DeviceConnectionType::FastUsbDevice => {
                // For USB devices, we need to find the device index
                // This is a limitation of the current design - we should improve this
                DeviceWorkerImpl::new_usb(self.thread_id, &device, 0, self.refresh_interval)?
            }
            pokeys_lib::DeviceConnectionType::NetworkDevice => {
                // For network devices, create a summary from the device info
                let device_summary = pokeys_lib::NetworkDeviceSummary {
                    serial_number,
                    ip_address: [0, 0, 0, 0], // This should be improved to get actual IP
                    host_ip: [0, 0, 0, 0],
                    firmware_version_major: device.device_data.firmware_version_major,
                    firmware_version_minor: device.device_data.firmware_version_minor,
                    firmware_revision: device.device_data.firmware_revision,
                    user_id: device.device_data.user_id,
                    dhcp: 0,
                    hw_type: 0,
                    use_udp: 0,
                };
                DeviceWorkerImpl::new_network(
                    self.thread_id,
                    &device,
                    device_summary,
                    self.refresh_interval,
                )?
            }
        };

        // Add logger if available
        if let Some(logger) = self.logger {
            worker = worker.with_logger(logger);
        }

        // Create a boxed worker
        let mut boxed_worker: Box<dyn DeviceWorker> = Box::new(worker);

        // Start the worker thread
        boxed_worker.start()?;

        Ok(boxed_worker)
    }
}
