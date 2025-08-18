#![allow(clippy::uninlined_format_args)]
//! Show device types for all connected devices
//!
//! This example discovers devices and shows their type information without performing any operations.

use log::info;
use pokeys_thread::{ThreadController, ThreadControllerBuilder};
use std::time::Duration;

fn main() {
    // Initialize logging
    env_logger::init();

    info!("=== PoKeys Device Type Display ===");

    // Create a thread controller
    let mut controller = ThreadControllerBuilder::new()
        .default_refresh_interval(100)
        .build();

    // Discover network devices
    match controller.discover_network_devices(3000) {
        Ok(devices) => {
            info!("Found {} network devices", devices.len());

            let mut device_threads = Vec::new();

            // Start threads for all devices
            for device_summary in devices {
                match controller.start_network_device_thread(device_summary.clone()) {
                    Ok(thread_id) => {
                        let description = format!(
                            "Network Device {} at {}.{}.{}.{}",
                            device_summary.serial_number,
                            device_summary.ip_address[0],
                            device_summary.ip_address[1],
                            device_summary.ip_address[2],
                            device_summary.ip_address[3]
                        );
                        device_threads.push((thread_id, description));
                        info!(
                            "Started thread {} for device {}",
                            thread_id, device_summary.serial_number
                        );
                    }
                    Err(e) => {
                        info!(
                            "Failed to start thread for device {}: {}",
                            device_summary.serial_number, e
                        );
                    }
                }
            }

            // Wait for all threads to initialize
            std::thread::sleep(Duration::from_millis(2000));

            // Display information about all connected devices
            info!("=== Device Information ===");
            for (thread_id, description) in &device_threads {
                match controller.get_state(*thread_id) {
                    Ok(state) => {
                        info!("Thread {}: {}", thread_id, description);
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

                        // Show raw device name for debugging
                        let device_name_str =
                            String::from_utf8_lossy(&state.device_data.device_name)
                                .trim_end_matches('\0')
                                .to_string();
                        info!("  Device Name (raw): '{}'", device_name_str);
                        info!(
                            "  Device Type ID: {} (0x{:X})",
                            state.device_data.device_type_id, state.device_data.device_type_id
                        );
                    }
                    Err(e) => {
                        info!("Failed to get state for thread {}: {}", thread_id, e);
                    }
                }
                info!(""); // Empty line for separation
            }

            // Stop all threads
            for (thread_id, _) in device_threads {
                if let Err(e) = controller.stop_thread(thread_id) {
                    info!("Failed to stop thread {}: {}", thread_id, e);
                }
            }
        }
        Err(e) => {
            info!("Failed to discover network devices: {}", e);
        }
    }

    info!("Device type display completed");
}
