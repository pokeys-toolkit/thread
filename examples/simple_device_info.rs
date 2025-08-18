#![allow(clippy::uninlined_format_args)]
//! Simple device information display
//!
//! This example just discovers devices and shows their information without trying to control them.

use log::info;
use pokeys_thread::{ThreadController, ThreadControllerBuilder};
use std::time::Duration;

fn main() {
    // Initialize logging
    env_logger::init();

    info!("Starting simple device info example");

    // Create a thread controller
    let mut controller = ThreadControllerBuilder::new()
        .default_refresh_interval(100)
        .build();

    // Discover network devices
    match controller.discover_network_devices(3000) {
        Ok(devices) => {
            info!("Found {} network devices", devices.len());

            for (i, device_summary) in devices.iter().enumerate() {
                info!("=== Network Device {} ===", i + 1);
                info!("Serial Number: {}", device_summary.serial_number);
                info!(
                    "IP Address: {}.{}.{}.{}",
                    device_summary.ip_address[0],
                    device_summary.ip_address[1],
                    device_summary.ip_address[2],
                    device_summary.ip_address[3]
                );
                info!(
                    "Firmware Version: {}.{}",
                    device_summary.firmware_version_major, device_summary.firmware_version_minor
                );

                // Start a thread for this device
                match controller.start_network_device_thread(device_summary.clone()) {
                    Ok(thread_id) => {
                        info!(
                            "Started thread {} for device {}",
                            thread_id, device_summary.serial_number
                        );

                        // Wait for thread to initialize
                        std::thread::sleep(Duration::from_millis(2000));

                        // Get detailed device state
                        match controller.get_state(thread_id) {
                            Ok(state) => {
                                info!(
                                    "Thread {}: Network Device {} at {}.{}.{}.{}",
                                    thread_id,
                                    state.device_data.serial_number,
                                    device_summary.ip_address[0],
                                    device_summary.ip_address[1],
                                    device_summary.ip_address[2],
                                    device_summary.ip_address[3]
                                );
                                info!("  Serial: {}", state.device_data.serial_number);
                                info!("  Type: {}", state.device_data.device_type_name());
                                info!(
                                    "  Firmware: {}.{}",
                                    state.device_data.firmware_version_major,
                                    state.device_data.firmware_version_minor
                                );
                                info!("  Pins: {}", state.device_info.pin_count);
                                info!("  PWM Channels: {}", state.device_info.pwm_count);
                                info!("  Analog Inputs: {}", state.device_info.analog_inputs);
                            }
                            Err(e) => {
                                info!("Failed to get device state for thread {}: {}", thread_id, e);
                            }
                        }

                        // Stop the thread
                        if let Err(e) = controller.stop_thread(thread_id) {
                            info!("Failed to stop thread {}: {}", thread_id, e);
                        }
                    }
                    Err(e) => {
                        info!(
                            "Failed to start thread for device {}: {}",
                            device_summary.serial_number, e
                        );
                    }
                }
            }
        }
        Err(e) => {
            info!("Failed to discover network devices: {}", e);
        }
    }

    info!("Simple device info completed");
}
