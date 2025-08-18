#![allow(clippy::uninlined_format_args)]
//! Error handling and recovery example
//!
//! This example demonstrates robust error handling in the threading system:
//! - Handling device connection failures
//! - Recovering from communication errors
//! - Monitoring thread health
//! - Implementing retry logic

use log::{error, info, warn};
use pokeys_thread::{DeviceOperations, ThreadController, ThreadControllerBuilder, ThreadStatus};
use std::thread;
use std::time::Duration;

fn main() {
    // Initialize logging
    env_logger::init();

    info!("Starting error handling and recovery example");

    // Create a thread controller
    let mut controller = ThreadControllerBuilder::new()
        .default_refresh_interval(100)
        .build();

    // Demonstrate device discovery error handling
    info!("=== Device Discovery Error Handling ===");

    let mut thread_id = None;
    let mut retry_count = 0;
    const MAX_RETRIES: u32 = 3;

    // Retry logic for device discovery
    while thread_id.is_none() && retry_count < MAX_RETRIES {
        retry_count += 1;
        info!(
            "Device discovery attempt {} of {}",
            retry_count, MAX_RETRIES
        );

        // Try USB devices first
        match controller.discover_usb_devices() {
            Ok(devices) => {
                info!("Found {} USB devices", devices.len());

                if !devices.is_empty() {
                    // Try to start a thread for the first device
                    match controller.start_usb_device_thread(devices[0]) {
                        Ok(tid) => {
                            info!(
                                "Successfully started thread {} for USB device {}",
                                tid, devices[0]
                            );
                            thread_id = Some(tid);
                            break;
                        }
                        Err(e) => {
                            error!(
                                "Failed to start USB device thread (attempt {}): {}",
                                retry_count, e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                error!(
                    "USB device discovery failed (attempt {}): {}",
                    retry_count, e
                );
            }
        }

        // If USB failed, try network devices
        if thread_id.is_none() {
            match controller.discover_network_devices(2000) {
                Ok(devices) => {
                    info!("Found {} network devices", devices.len());

                    if !devices.is_empty() {
                        match controller.start_network_device_thread(devices[0].clone()) {
                            Ok(tid) => {
                                info!(
                                    "Successfully started thread {} for network device {}",
                                    tid, devices[0].serial_number
                                );
                                thread_id = Some(tid);
                                break;
                            }
                            Err(e) => {
                                error!(
                                    "Failed to start network device thread (attempt {}): {}",
                                    retry_count, e
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "Network device discovery failed (attempt {}): {}",
                        retry_count, e
                    );
                }
            }
        }

        if thread_id.is_none() && retry_count < MAX_RETRIES {
            warn!("Attempt {} failed, retrying in 2 seconds...", retry_count);
            thread::sleep(Duration::from_millis(2000));
        }
    }

    if let Some(thread_id) = thread_id {
        info!(
            "Device connection established after {} attempts",
            retry_count
        );

        // Wait for thread to initialize
        thread::sleep(Duration::from_millis(500));

        // Demonstrate operation error handling
        info!("=== Operation Error Handling ===");

        // Test various operations with error handling
        test_digital_operations(&controller, thread_id);
        test_analog_operations(&controller, thread_id);
        test_pwm_operations(&controller, thread_id);

        // Demonstrate thread health monitoring
        info!("=== Thread Health Monitoring ===");
        monitor_thread_health(&controller, thread_id);

        // Demonstrate recovery from errors
        info!("=== Error Recovery ===");
        demonstrate_error_recovery(&mut controller, thread_id);

        // Clean shutdown
        info!("=== Clean Shutdown ===");
        match controller.stop_all() {
            Ok(_) => {
                info!("All threads stopped successfully");
            }
            Err(e) => {
                error!("Error during shutdown: {}", e);
                // Even if shutdown fails, we should continue cleanup
                warn!("Continuing with cleanup despite shutdown error");
            }
        }
    } else {
        error!(
            "Failed to connect to any device after {} attempts",
            MAX_RETRIES
        );
        info!("Please ensure a PoKeys device is connected and try again");
    }

    info!("Error handling and recovery example completed");
}

fn test_digital_operations(
    controller: &(impl ThreadController + DeviceOperations),
    thread_id: u32,
) {
    info!("Testing digital operations with error handling");

    // Test valid pin operations
    for pin in 1..=5 {
        match controller.set_digital_output(thread_id, pin, true) {
            Ok(_) => {
                info!("Successfully set digital output pin {} to HIGH", pin);

                // Try to read it back (if supported)
                match controller.get_digital_input(thread_id, pin) {
                    Ok(value) => {
                        info!(
                            "Digital input pin {} reads: {}",
                            pin,
                            if value { "HIGH" } else { "LOW" }
                        );
                    }
                    Err(e) => {
                        warn!("Could not read digital input pin {}: {}", pin, e);
                    }
                }

                // Set back to LOW
                if let Err(e) = controller.set_digital_output(thread_id, pin, false) {
                    error!("Failed to set pin {} back to LOW: {}", pin, e);
                }
            }
            Err(e) => {
                warn!("Failed to set digital output pin {}: {}", pin, e);
            }
        }
    }

    // Test invalid pin operations (should fail gracefully)
    info!("Testing invalid pin operations");
    match controller.set_digital_output(thread_id, 999, true) {
        Ok(_) => {
            warn!("Unexpectedly succeeded in setting invalid pin 999");
        }
        Err(e) => {
            info!("Expected error for invalid pin 999: {}", e);
        }
    }
}

fn test_analog_operations(controller: &(impl ThreadController + DeviceOperations), thread_id: u32) {
    info!("Testing analog operations with error handling");

    // Test analog input reading
    for channel in 1..=8 {
        match controller.get_analog_input(thread_id, channel) {
            Ok(value) => {
                let voltage = (value as f32 / 4095.0) * 5.0; // Assuming 5V reference
                info!(
                    "Analog input {}: {} (raw) / {:.3}V",
                    channel, value, voltage
                );
            }
            Err(e) => {
                warn!("Failed to read analog input {}: {}", channel, e);
            }
        }
    }

    // Test invalid analog channel
    match controller.get_analog_input(thread_id, 999) {
        Ok(value) => {
            warn!(
                "Unexpectedly read value {} from invalid analog channel 999",
                value
            );
        }
        Err(e) => {
            info!("Expected error for invalid analog channel 999: {}", e);
        }
    }
}

fn test_pwm_operations(controller: &(impl ThreadController + DeviceOperations), thread_id: u32) {
    info!("Testing PWM operations with error handling");

    // Test PWM operations
    for channel in 0..4 {
        match controller.set_pwm_duty_cycle_percent(thread_id, channel, 50.0) {
            Ok(_) => {
                info!("Successfully set PWM channel {} to 50%", channel);

                // Clear it
                if let Err(e) = controller.set_pwm_duty_cycle_percent(thread_id, channel, 0.0) {
                    error!("Failed to clear PWM channel {}: {}", channel, e);
                }
            }
            Err(e) => {
                warn!("Failed to set PWM channel {}: {}", channel, e);
            }
        }
    }

    // Test invalid PWM values
    match controller.set_pwm_duty_cycle_percent(thread_id, 0, 150.0) {
        Ok(_) => {
            warn!("Unexpectedly accepted invalid PWM duty cycle 150%");
        }
        Err(e) => {
            info!("Expected error for invalid PWM duty cycle 150%: {}", e);
        }
    }
}

fn monitor_thread_health(controller: &impl ThreadController, thread_id: u32) {
    info!("Monitoring thread health for 5 seconds");

    for i in 0..5 {
        thread::sleep(Duration::from_millis(1000));

        match controller.get_status(thread_id) {
            Ok(status) => match status {
                ThreadStatus::Running => {
                    info!(
                        "Health check {}: Thread {} is running normally",
                        i + 1,
                        thread_id
                    );
                }
                ThreadStatus::Stopped => {
                    error!(
                        "Health check {}: Thread {} has stopped unexpectedly!",
                        i + 1,
                        thread_id
                    );
                    break;
                }
                ThreadStatus::Error => {
                    error!(
                        "Health check {}: Thread {} is in error state!",
                        i + 1,
                        thread_id
                    );
                    break;
                }
                ThreadStatus::Paused => {
                    warn!("Health check {}: Thread {} is paused", i + 1, thread_id);
                }
            },
            Err(e) => {
                error!("Health check {}: Failed to get thread status: {}", i + 1, e);
                break;
            }
        }

        // Also check if thread is running using convenience method
        match controller.is_thread_running(thread_id) {
            Ok(is_running) => {
                if !is_running {
                    error!(
                        "Health check {}: Thread {} is not running!",
                        i + 1,
                        thread_id
                    );
                    break;
                }
            }
            Err(e) => {
                error!(
                    "Health check {}: Failed to check if thread is running: {}",
                    i + 1,
                    e
                );
            }
        }
    }
}

fn demonstrate_error_recovery(
    controller: &mut (impl ThreadController + DeviceOperations),
    thread_id: u32,
) {
    info!("Demonstrating error recovery scenarios");

    // Simulate recovery by checking thread status and attempting restart if needed
    match controller.get_status(thread_id) {
        Ok(status) => {
            match status {
                ThreadStatus::Running => {
                    info!("Thread is healthy, no recovery needed");
                }
                ThreadStatus::Stopped | ThreadStatus::Error => {
                    warn!("Thread is in bad state: {:?}, attempting recovery", status);

                    // In a real application, you might:
                    // 1. Stop the problematic thread
                    // 2. Wait a bit
                    // 3. Restart the thread
                    // 4. Verify it's working

                    info!("Recovery would involve restarting the thread");
                    info!("For this example, we'll just log the recovery steps");
                }
                ThreadStatus::Paused => {
                    info!("Thread is paused, may need to resume");
                }
            }
        }
        Err(e) => {
            error!("Cannot determine thread status for recovery: {}", e);
        }
    }

    // Demonstrate graceful degradation
    info!("Testing graceful degradation");

    // Try an operation that might fail
    match controller.set_digital_output(thread_id, 1, true) {
        Ok(_) => {
            info!("Operation succeeded");
        }
        Err(e) => {
            warn!("Operation failed, implementing fallback: {}", e);
            // In a real application, you might:
            // - Use a different pin
            // - Skip the operation
            // - Use a different device
            // - Alert the user
            info!("Fallback: Operation skipped due to error");
        }
    }
}
