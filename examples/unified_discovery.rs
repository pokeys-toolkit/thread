#![allow(clippy::uninlined_format_args)]
//! Unified device discovery example
//!
//! This example demonstrates how to discover and connect to both USB and network PoKeys devices
//! using a unified approach that tries USB first, then falls back to network discovery.

use log::{error, info, warn};
use pokeys_thread::{ThreadController, ThreadControllerBuilder};
use std::thread;
use std::time::Duration;

/// Helper function to discover and start a device thread
/// Returns the thread ID if successful, None otherwise
fn discover_and_start_device(controller: &mut impl ThreadController) -> Option<u32> {
    // Try USB devices first
    match controller.discover_usb_devices() {
        Ok(devices) => {
            info!("Found {} USB devices", devices.len());

            if !devices.is_empty() {
                // Start a thread for the first USB device
                match controller.start_usb_device_thread(devices[0]) {
                    Ok(thread_id) => {
                        info!("Started thread {} for USB device {}", thread_id, devices[0]);
                        return Some(thread_id);
                    }
                    Err(e) => {
                        error!(
                            "Failed to start thread for USB device {}: {}",
                            devices[0], e
                        );
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to discover USB devices: {}", e);
        }
    }

    // If no USB device was started, try network devices
    match controller.discover_network_devices(2000) {
        Ok(devices) => {
            info!("Found {} network devices", devices.len());

            if !devices.is_empty() {
                // Start a thread for the first network device
                match controller.start_network_device_thread(devices[0].clone()) {
                    Ok(thread_id) => {
                        info!(
                            "Started thread {} for network device with serial {}",
                            thread_id, devices[0].serial_number
                        );
                        return Some(thread_id);
                    }
                    Err(e) => {
                        error!(
                            "Failed to start thread for network device {}: {}",
                            devices[0].serial_number, e
                        );
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to discover network devices: {}", e);
        }
    }

    None
}

fn main() {
    // Initialize logging
    env_logger::init();

    info!("Starting unified discovery example");

    // Create a thread controller
    let mut controller = ThreadControllerBuilder::new()
        .default_refresh_interval(100)
        .build();

    // Discover and start a device
    if let Some(thread_id) = discover_and_start_device(&mut controller) {
        // Wait a bit for the thread to start
        thread::sleep(Duration::from_millis(100));

        // Get the device state
        match controller.get_state(thread_id) {
            Ok(state) => {
                info!(
                    "Connected to device: Serial {}, FW {}.{}",
                    state.device_data.serial_number,
                    state.device_data.firmware_version_major,
                    state.device_data.firmware_version_minor
                );
                info!("Device type: {}", state.device_data.device_type_name());
                info!("Build date: {}", state.device_data.build_date_string());
            }
            Err(e) => {
                error!("Failed to get device state: {}", e);
            }
        }

        // Check the status
        match controller.get_status(thread_id) {
            Ok(status) => {
                info!("Thread {} status: {:?}", thread_id, status);
            }
            Err(e) => {
                error!("Failed to get thread status: {}", e);
            }
        }

        // Wait for user input to exit
        info!("Press Enter to exit...");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        // Stop the thread
        match controller.stop_all() {
            Ok(_) => {
                info!("Device thread stopped");
            }
            Err(e) => {
                error!("Failed to stop thread: {}", e);
            }
        }
    } else {
        warn!("No PoKeys devices found (USB or network)");
        info!("Make sure a PoKeys device is connected via USB or available on the network");
    }

    info!("Unified discovery example completed");
}
