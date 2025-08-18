#![allow(clippy::uninlined_format_args)]
//! Multi-device management example
//!
//! This example demonstrates how to manage multiple PoKeys devices simultaneously,
//! showing how to:
//! - Discover and connect to multiple devices
//! - Manage threads for each device
//! - Perform operations on multiple devices concurrently
//! - Monitor the status of all devices

use log::{error, info, warn};
use pokeys_lib::PinFunction;
use pokeys_thread::{DeviceOperations, ThreadController, ThreadControllerBuilder};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

fn main() {
    // Initialize logging
    env_logger::init();

    info!("Starting multi-device management example");

    // Create a thread controller
    let mut controller = ThreadControllerBuilder::new()
        .default_refresh_interval(100)
        .build();

    let mut device_threads: HashMap<u32, String> = HashMap::new(); // thread_id -> device_description

    // Discover and start USB devices
    // match controller.discover_usb_devices() {
    //     Ok(devices) => {
    //         info!("Found {} USB devices", devices.len());

    //         for device_index in devices {
    //             match controller.start_usb_device_thread(device_index) {
    //                 Ok(thread_id) => {
    //                     let description = format!("USB Device {}", device_index);
    //                     info!("Started thread {} for {}", thread_id, description);
    //                     device_threads.insert(thread_id, description);
    //                 }
    //                 Err(e) => {
    //                     error!("Failed to start thread for USB device {}: {}", device_index, e);
    //                 }
    //             }
    //         }
    //     }
    //     Err(e) => {
    //         error!("Failed to discover USB devices: {}", e);
    //     }
    // }

    // Discover and start network devices
    match controller.discover_network_devices(2000) {
        Ok(devices) => {
            info!("Found {} network devices", devices.len());

            for device in devices {
                let ip_str = format!(
                    "{}.{}.{}.{}",
                    device.ip_address[0],
                    device.ip_address[1],
                    device.ip_address[2],
                    device.ip_address[3]
                );

                match controller.start_network_device_thread(device.clone()) {
                    Ok(thread_id) => {
                        let description =
                            format!("Network Device {} at {}", device.serial_number, ip_str);
                        info!("Started thread {} for {}", thread_id, description);
                        device_threads.insert(thread_id, description);
                    }
                    Err(e) => {
                        error!(
                            "Failed to start thread for network device {}: {}",
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

    if device_threads.is_empty() {
        warn!("No PoKeys devices found");
        return;
    }

    info!(
        "Successfully started {} device threads",
        device_threads.len()
    );

    // Wait for all threads to initialize
    thread::sleep(Duration::from_millis(500));

    // Display information about all connected devices
    info!("=== Connected Devices ===");
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
            }
            Err(e) => {
                error!("Failed to get state for thread {}: {}", thread_id, e);
            }
        }
    }

    // Demonstrate concurrent operations on all devices
    info!("=== Performing concurrent operations ===");

    // Configure pins before using them
    info!("=== Pin Configuration ===");
    for (thread_id, description) in &device_threads {
        info!("Configuring pins on {}", description);

        // Configure pins 2-5 as digital outputs (avoiding pin 1 which is often reserved)
        for pin in 2..=5 {
            if let Err(e) = controller.set_pin_function(*thread_id, pin, PinFunction::DigitalOutput)
            {
                warn!(
                    "Failed to configure pin {} on thread {}: {}",
                    pin, thread_id, e
                );
            } else {
                info!(
                    "Configured pin {} as digital output on {}",
                    pin, description
                );
                break; // Use the first pin that configures successfully
            }
        }
    }

    // Wait for pin configuration to take effect
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Set digital outputs on all devices
    for (thread_id, description) in &device_threads {
        info!("Setting digital output on {}", description);

        // Try pins 2-5 to find one that works (avoiding pin 1 which is often reserved)
        let mut success = false;
        for pin in 2..=5 {
            if controller.set_digital_output(*thread_id, pin, true).is_ok() {
                info!("Successfully set pin {} to HIGH on {}", pin, description);
                success = true;
                break;
            }
        }

        if !success {
            error!("Failed to set any digital output on thread {}", thread_id);
        }
    }

    // Wait a bit
    thread::sleep(Duration::from_millis(1000));

    // Read digital inputs from all devices
    for (thread_id, description) in &device_threads {
        info!("Reading digital input from {}", description);

        match controller.get_digital_input(*thread_id, 2) {
            Ok(value) => {
                info!("  Pin 2: {}", if value { "HIGH" } else { "LOW" });
            }
            Err(e) => {
                error!(
                    "Failed to read digital input from thread {}: {}",
                    thread_id, e
                );
            }
        }
    }

    // Monitor thread status
    info!("=== Thread Status Monitoring ===");
    let active_threads = controller.list_active_threads().unwrap_or_default();
    info!("Active threads: {:?}", active_threads);

    for thread_id in &active_threads {
        if let Ok(status) = controller.get_status(*thread_id) {
            let description = device_threads
                .get(thread_id)
                .unwrap_or(&"Unknown".to_string())
                .clone();
            info!("Thread {} ({}): {:?}", thread_id, description, status);

            // Check if thread is running using convenience method
            if let Ok(is_running) = controller.is_thread_running(*thread_id) {
                info!("  Running: {}", is_running);
            }
        }
    }

    // Set all outputs back to LOW
    info!("=== Cleanup ===");
    for (thread_id, description) in &device_threads {
        info!("Cleaning up {}", description);

        // Try pins 2-5 to turn them off
        for pin in 2..=5 {
            if controller
                .set_digital_output(*thread_id, pin, false)
                .is_ok()
            {
                info!("Successfully set pin {} to LOW on {}", pin, description);
            }
        }
    }

    // Wait for user input to exit
    info!("Press Enter to stop all devices...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // Stop all threads
    match controller.stop_all() {
        Ok(_) => {
            info!(
                "All {} device threads stopped successfully",
                device_threads.len()
            );
        }
        Err(e) => {
            error!("Failed to stop all threads: {}", e);
        }
    }

    info!("Multi-device management example completed");
}
