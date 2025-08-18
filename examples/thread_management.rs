#![allow(clippy::uninlined_format_args)]
//! Thread management example
//!
//! This example demonstrates the thread management capabilities including:
//! - Starting and stopping individual threads
//! - Checking thread status with convenience methods
//! - Listing active threads
//! - Managing thread lifecycle

use log::{error, info, warn};
use pokeys_thread::{ThreadController, ThreadControllerBuilder, ThreadStatus};
use std::thread;
use std::time::Duration;

fn main() {
    // Initialize logging
    env_logger::init();

    info!("Starting thread management example");

    // Create a thread controller
    let mut controller = ThreadControllerBuilder::new()
        .default_refresh_interval(100)
        .build();

    // Discover devices
    let mut available_devices = Vec::new();

    // Check for USB devices
    match controller.discover_usb_devices() {
        Ok(devices) => {
            info!("Found {} USB devices", devices.len());
            for device_index in devices {
                available_devices.push(format!("USB:{}", device_index));
            }
        }
        Err(e) => {
            error!("Failed to discover USB devices: {}", e);
        }
    }

    // Check for network devices
    match controller.discover_network_devices(2000) {
        Ok(devices) => {
            info!("Found {} network devices", devices.len());
            for device in devices {
                available_devices.push(format!("NET:{}", device.serial_number));
            }
        }
        Err(e) => {
            error!("Failed to discover network devices: {}", e);
        }
    }

    if available_devices.is_empty() {
        warn!("No devices found for thread management demonstration");
        return;
    }

    info!("Available devices: {:?}", available_devices);

    // Start threads for all devices
    let mut started_threads = Vec::new();

    // Start USB device threads
    if let Ok(usb_devices) = controller.discover_usb_devices() {
        for device_index in usb_devices {
            match controller.start_usb_device_thread(device_index) {
                Ok(thread_id) => {
                    info!(
                        "Started thread {} for USB device {}",
                        thread_id, device_index
                    );
                    started_threads.push(thread_id);
                }
                Err(e) => {
                    error!("Failed to start USB device thread: {}", e);
                }
            }
        }
    }

    // Start network device threads
    if let Ok(network_devices) = controller.discover_network_devices(1000) {
        for device in network_devices {
            match controller.start_network_device_thread(device.clone()) {
                Ok(thread_id) => {
                    info!(
                        "Started thread {} for network device {}",
                        thread_id, device.serial_number
                    );
                    started_threads.push(thread_id);
                }
                Err(e) => {
                    error!("Failed to start network device thread: {}", e);
                }
            }
        }
    }

    if started_threads.is_empty() {
        warn!("No threads were started");
        return;
    }

    // Wait for threads to initialize
    thread::sleep(Duration::from_millis(500));

    // Demonstrate thread management features
    info!("=== Thread Management Demonstration ===");

    // 1. List all active threads
    match controller.list_active_threads() {
        Ok(active_threads) => {
            info!("Active threads: {:?}", active_threads);
            info!("Total active threads: {}", active_threads.len());
        }
        Err(e) => {
            error!("Failed to list active threads: {}", e);
        }
    }

    // 2. Check status of each thread
    info!("=== Thread Status Check ===");
    for thread_id in &started_threads {
        // Using the convenience method
        match controller.is_thread_running(*thread_id) {
            Ok(is_running) => {
                info!("Thread {} is running: {}", thread_id, is_running);
            }
            Err(e) => {
                error!("Failed to check if thread {} is running: {}", thread_id, e);
            }
        }

        // Using the detailed status method
        match controller.get_status(*thread_id) {
            Ok(status) => {
                info!("Thread {} detailed status: {:?}", thread_id, status);
            }
            Err(e) => {
                error!("Failed to get status for thread {}: {}", thread_id, e);
            }
        }

        // Get device information
        match controller.get_state(*thread_id) {
            Ok(state) => {
                info!("Thread {} device info:", thread_id);
                info!("  Serial: {}", state.device_data.serial_number);
                info!("  Type: {}", state.device_data.device_type_name());
                info!("  Pins: {}", state.device_info.pin_count);
            }
            Err(e) => {
                error!("Failed to get state for thread {}: {}", thread_id, e);
            }
        }
    }

    // 3. Demonstrate stopping individual threads
    if started_threads.len() > 1 {
        info!("=== Individual Thread Management ===");
        let thread_to_stop = started_threads[0];

        info!("Stopping thread {} individually", thread_to_stop);
        match controller.stop_thread(thread_to_stop) {
            Ok(_) => {
                info!("Successfully stopped thread {}", thread_to_stop);

                // Verify it's stopped
                thread::sleep(Duration::from_millis(100));
                match controller.is_thread_running(thread_to_stop) {
                    Ok(is_running) => {
                        info!("Thread {} is now running: {}", thread_to_stop, is_running);
                    }
                    Err(e) => {
                        // This is expected if the thread was removed
                        info!("Thread {} no longer exists: {}", thread_to_stop, e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to stop thread {}: {}", thread_to_stop, e);
            }
        }

        // List active threads again
        match controller.list_active_threads() {
            Ok(active_threads) => {
                info!("Active threads after stopping one: {:?}", active_threads);
            }
            Err(e) => {
                error!("Failed to list active threads: {}", e);
            }
        }
    }

    // 4. Monitor remaining threads for a bit
    info!("=== Thread Monitoring ===");
    for i in 0..5 {
        info!("Monitoring cycle {}", i + 1);

        match controller.list_active_threads() {
            Ok(active_threads) => {
                for thread_id in active_threads {
                    if let Ok(status) = controller.get_status(thread_id) {
                        match status {
                            ThreadStatus::Running => {
                                info!("  Thread {} is running normally", thread_id);
                            }
                            ThreadStatus::Stopped => {
                                info!("  Thread {} has stopped", thread_id);
                            }
                            ThreadStatus::Error => {
                                warn!("  Thread {} has an error", thread_id);
                            }
                            ThreadStatus::Paused => {
                                info!("  Thread {} is paused", thread_id);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to list active threads: {}", e);
            }
        }

        thread::sleep(Duration::from_millis(1000));
    }

    // 5. Stop all remaining threads
    info!("=== Stopping All Threads ===");
    match controller.stop_all() {
        Ok(_) => {
            info!("All threads stopped successfully");
        }
        Err(e) => {
            error!("Failed to stop all threads: {}", e);
        }
    }

    // Verify all threads are stopped
    thread::sleep(Duration::from_millis(100));
    match controller.list_active_threads() {
        Ok(active_threads) => {
            if active_threads.is_empty() {
                info!("Confirmed: No active threads remaining");
            } else {
                warn!(
                    "Warning: {} threads still active: {:?}",
                    active_threads.len(),
                    active_threads
                );
            }
        }
        Err(e) => {
            error!("Failed to verify thread cleanup: {}", e);
        }
    }

    info!("Thread management example completed");
}
