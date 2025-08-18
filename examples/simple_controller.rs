#![allow(clippy::uninlined_format_args)]
//! Simple example of using the thread controller

use log::{error, info};
use pokeys_thread::{ThreadController, ThreadControllerBuilder};
use std::thread;
use std::time::Duration;

fn main() {
    // Initialize logging
    env_logger::init();

    info!("Starting simple controller example");

    // Create a thread controller
    let mut controller = ThreadControllerBuilder::new()
        .default_refresh_interval(100)
        .build();

    // Discover USB devices
    match controller.discover_usb_devices() {
        Ok(devices) => {
            info!("Found {} USB devices", devices.len());

            // Start a thread for each device
            for device_index in devices {
                match controller.start_usb_device_thread(device_index) {
                    Ok(thread_id) => {
                        info!(
                            "Started thread {} for USB device {}",
                            thread_id, device_index
                        );

                        // Wait a bit for the thread to start
                        thread::sleep(Duration::from_millis(100));

                        // Check the status
                        match controller.get_status(thread_id) {
                            Ok(status) => {
                                info!("Thread {} status: {:?}", thread_id, status);
                            }
                            Err(e) => {
                                error!("Failed to get thread status: {}", e);
                            }
                        }

                        // Get the state
                        match controller.get_state(thread_id) {
                            Ok(state) => {
                                info!(
                                    "Thread {} device: Serial {}, FW {}.{}",
                                    thread_id,
                                    state.device_data.serial_number,
                                    state.device_data.firmware_version_major,
                                    state.device_data.firmware_version_minor
                                );
                            }
                            Err(e) => {
                                error!("Failed to get thread state: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to start thread for device {}: {}", device_index, e);
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to discover USB devices: {}", e);
        }
    }

    // Discover network devices
    match controller.discover_network_devices(2000) {
        Ok(devices) => {
            info!("Found {} network devices", devices.len());

            // Start a thread for each device
            for device in devices {
                match controller.start_network_device_thread(device.clone()) {
                    Ok(thread_id) => {
                        info!(
                            "Started thread {} for network device with serial {}",
                            thread_id, device.serial_number
                        );

                        // Wait a bit for the thread to start
                        thread::sleep(Duration::from_millis(100));

                        // Check the status
                        match controller.get_status(thread_id) {
                            Ok(status) => {
                                info!("Thread {} status: {:?}", thread_id, status);
                            }
                            Err(e) => {
                                error!("Failed to get thread status: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to start thread for device with serial {}: {}",
                            device.serial_number, e
                        );
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to discover network devices: {}", e);
        }
    }

    // Wait for user input to exit
    info!("Press Enter to exit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // Stop all threads
    match controller.stop_all() {
        Ok(_) => {
            info!("All threads stopped");
        }
        Err(e) => {
            error!("Failed to stop all threads: {}", e);
        }
    }
}
