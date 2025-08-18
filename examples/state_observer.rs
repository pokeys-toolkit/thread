#![allow(clippy::uninlined_format_args)]
//! Example of using the state observer

use log::{error, info};
use pokeys_thread::{DeviceCommand, StateChangeType, ThreadController, ThreadControllerBuilder};
use std::thread;
use std::time::Duration;

fn main() {
    // Initialize logging
    env_logger::init();

    info!("Starting state observer example");

    // Create a thread controller
    let mut controller = ThreadControllerBuilder::new()
        .default_refresh_interval(100)
        .build();

    let mut thread_id = None;

    // Try USB devices first
    match controller.discover_usb_devices() {
        Ok(devices) => {
            info!("Found {} USB devices", devices.len());

            if !devices.is_empty() {
                // Start a thread for the first USB device
                match controller.start_usb_device_thread(devices[0]) {
                    Ok(tid) => {
                        info!("Started thread {} for USB device {}", tid, devices[0]);
                        thread_id = Some(tid);
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
    if thread_id.is_none() {
        match controller.discover_network_devices(2000) {
            Ok(devices) => {
                info!("Found {} network devices", devices.len());

                if !devices.is_empty() {
                    // Start a thread for the first network device
                    match controller.start_network_device_thread(devices[0].clone()) {
                        Ok(tid) => {
                            info!(
                                "Started thread {} for network device with serial {}",
                                tid, devices[0].serial_number
                            );
                            thread_id = Some(tid);
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
    }

    if let Some(thread_id) = thread_id {
        // Wait a bit for the thread to start
        thread::sleep(Duration::from_millis(100));

        // Create a state observer
        match controller.create_observer(thread_id) {
            Ok(observer) => {
                info!("Created state observer for thread {}", thread_id);

                // Start a thread to monitor state changes
                let monitor_thread = thread::spawn(move || {
                    info!("State monitor thread started");

                    // Monitor state changes for 10 seconds
                    let start_time = std::time::Instant::now();
                    while start_time.elapsed() < Duration::from_secs(60) {
                        // Wait for a state change with a timeout
                        if let Some(change_type) =
                            observer.wait_for_change(Duration::from_millis(500))
                        {
                            match change_type {
                                StateChangeType::DigitalInput { pin, value } => {
                                    info!("Digital input {} changed to {}", pin, value);
                                }
                                StateChangeType::DigitalOutput { pin, value } => {
                                    info!("Digital output {} changed to {}", pin, value);
                                }
                                StateChangeType::AnalogInput { pin, value } => {
                                    info!("Analog input {} changed to {}", pin, value);
                                }
                                StateChangeType::AnalogOutput { pin, value } => {
                                    info!("Analog output {} changed to {}", pin, value);
                                }
                                StateChangeType::EncoderValue { index, value } => {
                                    info!("Encoder {} changed to {}", index, value);
                                }
                                StateChangeType::PwmDutyCycle { channel, duty } => {
                                    info!("PWM channel {} duty changed to {}", channel, duty);
                                }
                                StateChangeType::ThreadStatus { status } => {
                                    info!("Thread status changed to {:?}", status);
                                }
                                StateChangeType::Error { message } => {
                                    if let Some(msg) = message {
                                        info!("Error occurred: {}", msg);
                                    } else {
                                        info!("Error cleared");
                                    }
                                }
                                StateChangeType::CustomValue { key, value } => {
                                    info!("Custom value {} changed to {}", key, value);
                                }
                                StateChangeType::FullUpdate => {
                                    info!("Full state update");
                                }
                            }
                        }
                    }

                    info!("State monitor thread finished");
                });

                // Set a custom value
                let shared_state = controller.get_shared_state(thread_id).unwrap();
                shared_state.set_custom_value("example", "Hello, world!");

                // Wait for the monitor thread to finish
                monitor_thread.join().unwrap();

                // Stop the thread
                match controller.send_command(thread_id, DeviceCommand::Terminate) {
                    Ok(_) => {
                        info!("Sent terminate command to thread {}", thread_id);
                    }
                    Err(e) => {
                        error!("Failed to send terminate command: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to create state observer: {}", e);
            }
        }
    } else {
        info!("No PoKeys devices found (USB or network)");
    }

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
