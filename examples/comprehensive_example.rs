#![allow(clippy::uninlined_format_args)]
//! # Comprehensive Example
//!
//! This example demonstrates all the features of the pokeys-thread crate.
//! It shows how to:
//! - Create a thread controller
//! - Discover devices
//! - Start device threads
//! - Perform device operations
//! - Monitor state changes
//! - Configure logging
//! - Handle errors

use log::{debug, error, info, warn, LevelFilter};
use pokeys_lib::PinFunction;
use pokeys_thread::{
    DeviceCommand, DeviceOperations, SimpleLogger, ThreadController, ThreadControllerBuilder,
    ThreadStatus,
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    // Initialize the logger
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();

    info!("Starting comprehensive example");

    // Create a simple logger
    let logger = Arc::new(SimpleLogger::new(LevelFilter::Debug));

    // Create a thread controller with the logger
    let mut controller = ThreadControllerBuilder::new()
        .default_refresh_interval(100)
        .with_logger(logger.clone())
        .build();

    let mut thread_id = None;

    // Try USB devices first
    // match controller.discover_usb_devices() {
    //     Ok(devices) => {
    //         info!("Found {} USB devices", devices.len());

    //         if !devices.is_empty() {
    //             // Start a thread for the first USB device
    //             let device_index = devices[0];
    //             match controller.start_usb_device_thread(device_index) {
    //                 Ok(id) => {
    //                     info!("Started thread {} for USB device {}", id, device_index);
    //                     thread_id = Some(id);
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

    // If no USB device was started, try network devices
    if thread_id.is_none() {
        match controller.discover_network_devices(2000) {
            Ok(devices) => {
                info!("Found {} network devices", devices.len());

                if !devices.is_empty() {
                    // Start a thread for the first network device
                    match controller.start_network_device_thread(devices[0].clone()) {
                        Ok(id) => {
                            info!(
                                "Started thread {} for network device with serial {}",
                                id, devices[0].serial_number
                            );
                            thread_id = Some(id);
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
        // Wait for the thread to initialize
        thread::sleep(Duration::from_millis(500));

        // Get the device state
        match controller.get_state(thread_id) {
            Ok(state) => {
                info!("Device state:");
                info!("  Serial number: {}", state.device_data.serial_number);
                info!(
                    "  Software version: {} (parsed from bytes 5-6)",
                    state.device_data.software_version_string()
                );
                info!(
                    "  Firmware version: {}.{} (from bytes 17-18)",
                    state.device_data.secondary_firmware_version_major,
                    state.device_data.secondary_firmware_version_minor
                );
                info!("  Device type: {}", state.device_data.device_type_name());
                info!(
                    "  Device name: {}",
                    String::from_utf8_lossy(&state.device_data.device_name).trim_end_matches('\0')
                );
                debug!(
                    "  Build date: {}",
                    String::from_utf8_lossy(&state.device_data.build_date).trim_end_matches('\0')
                );
                debug!(
                    "  HW type: {}, FW type: {}, Product ID: {}",
                    state.device_data.hw_type,
                    state.device_data.fw_type,
                    state.device_data.product_id
                );
                debug!(
                    "  User ID: {}, Device type: {}",
                    state.device_data.user_id, state.device_data.device_type
                );
                debug!(
                    "  Raw firmware bytes: major={} (byte 5), minor={} (byte 6)",
                    state.device_data.firmware_version_major,
                    state.device_data.firmware_version_minor
                );
                debug!("  Pins: {} configured", state.pins.len());
                debug!("  Encoders: {} configured", state.encoders.len());
            }
            Err(e) => {
                error!("Failed to get device state: {}", e);
            }
        }

        // Create a state observer
        // let observer = match controller.create_observer(thread_id) {
        //     Ok(observer) => {
        //         info!("Created state observer for thread {}", thread_id);
        //         observer
        //     }
        //     Err(e) => {
        //         error!("Failed to create state observer: {}", e);
        //         return;
        //     }
        // };

        // Set log level to Info
        if let Err(e) = controller.set_thread_log_level(thread_id, LevelFilter::Info) {
            error!("Failed to set thread log level: {}", e);
        }

        // Configure pins before using them
        info!("Configuring pins for digital output");
        // for i in 0..5 {
        //     let pin = 1 + i;
        if let Err(e) = controller.set_pin_function(thread_id, 1, PinFunction::DigitalOutput) {
            error!("Failed to configure pin {} as digital output: {}", 1, e);
        } else {
            info!("Successfully configured pin {} as digital output", 1);
        }
        // }

        // Wait a bit for pin configuration to take effect
        thread::sleep(Duration::from_millis(500));

        // Toggle some digital outputs
        // info!("Toggling digital outputs");
        // for i in 0..5 {
        //     let pin = 1 + i;

        //     // Set output high
        //     if let Err(e) = controller.set_digital_output(thread_id, pin, true) {
        //         error!("Failed to set digital output {} high: {}", pin, e);
        //     }

        //     thread::sleep(Duration::from_millis(200));

        //     // Set output low
        //     if let Err(e) = controller.set_digital_output(thread_id, pin, false) {
        //         error!("Failed to set digital output {} low: {}", pin, e);
        //     }

        //     thread::sleep(Duration::from_millis(200));
        // }

        // Set PWM duty cycles
        // info!("Setting PWM duty cycles");
        // for i in 0..3 {
        //     let channel = i;
        //     let duty_percent = 25.0 * (i + 1) as f32;

        //     if let Err(e) = controller.set_pwm_duty_cycle_percent(thread_id, channel, duty_percent) {
        //         error!(
        //             "Failed to set PWM channel {} duty to {}%: {}",
        //             channel, duty_percent, e
        //         );
        //     }

        //     thread::sleep(Duration::from_millis(200));
        // }

        // Configure encoders
        // info!("Configuring encoders");
        // for i in 0..2 {
        //     let encoder_index = i;
        //     let pin_a = 1 + i * 2;
        //     let pin_b = 2 + i * 2;

        //     if let Err(e) =
        //         controller.configure_encoder(thread_id, encoder_index, pin_a, pin_b, true, true)
        //     {
        //         error!("Failed to configure encoder {}: {}", encoder_index, e);
        //     }

        //     thread::sleep(Duration::from_millis(200));
        // }

        // Read inputs
        // info!("Reading inputs");
        // for i in 0..5 {
        //     let pin = 1 + i;

        //     match controller.get_digital_input(thread_id, pin) {
        //         Ok(value) => {
        //             info!("Digital input {} value: {}", pin, value);
        //         }
        //         Err(e) => {
        //             error!("Failed to get digital input {}: {}", pin, e);
        //         }
        //     }

        //     thread::sleep(Duration::from_millis(100));
        // }

        // // Read analog inputs
        // info!("Reading analog inputs");
        // for i in 0..3 {
        //     let pin = 1 + i;

        //     match controller.get_analog_input(thread_id, pin) {
        //         Ok(value) => {
        //             info!("Analog input {} value: {}", pin, value);
        //         }
        //         Err(e) => {
        //             error!("Failed to get analog input {}: {}", pin, e);
        //         }
        //     }

        //     thread::sleep(Duration::from_millis(100));
        // }

        // // Read encoder values
        // info!("Reading encoder values");
        // for i in 0..2 {
        //     let encoder_index = i;

        //     match controller.get_encoder_value(thread_id, encoder_index) {
        //         Ok(value) => {
        //             info!("Encoder {} value: {}", encoder_index, value);
        //         }
        //         Err(e) => {
        //             error!("Failed to get encoder {} value: {}", encoder_index, e);
        //         }
        //     }

        //     thread::sleep(Duration::from_millis(100));
        // }

        // Process state changes
        // info!("Processing state changes");
        // observer.process_all_changes(|change| match change {
        //     StateChangeType::DigitalInput { pin, value } => {
        //         info!("Digital input {} changed to {}", pin, value);
        //     }
        //     StateChangeType::DigitalOutput { pin, value } => {
        //         info!("Digital output {} changed to {}", pin, value);
        //     }
        //     StateChangeType::AnalogInput { pin, value } => {
        //         info!("Analog input {} changed to {}", pin, value);
        //     }
        //     StateChangeType::AnalogOutput { pin, value } => {
        //         info!("Analog output {} changed to {}", pin, value);
        //     }
        //     StateChangeType::EncoderValue { index, value } => {
        //         info!("Encoder {} changed to {}", index, value);
        //     }
        //     StateChangeType::PwmDutyCycle { channel, duty } => {
        //         info!("PWM channel {} duty changed to {}", channel, duty);
        //     }
        //     StateChangeType::ThreadStatus { status } => {
        //         info!("Thread status changed to {:?}", status);
        //     }
        //     StateChangeType::Error { message } => {
        //         if let Some(msg) = message {
        //             error!("Error occurred: {}", msg);
        //         } else {
        //             info!("Error cleared");
        //         }
        //     }
        //     StateChangeType::CustomValue { key, value } => {
        //         info!("Custom value {} changed to {}", key, value);
        //     }
        //     StateChangeType::FullUpdate => {
        //         debug!("Full state update");
        //     }
        // });

        // // Test thread control commands
        // info!("Testing thread control commands");

        // // Pause the thread
        // if let Err(e) = controller.send_command(thread_id, DeviceCommand::Pause) {
        //     error!("Failed to pause thread: {}", e);
        // }

        // thread::sleep(Duration::from_millis(500));

        // // Check the status
        // match controller.get_status(thread_id) {
        //     Ok(status) => {
        //         info!("Thread status: {:?}", status);
        //         assert_eq!(status, ThreadStatus::Paused);
        //     }
        //     Err(e) => {
        //         error!("Failed to get thread status: {}", e);
        //     }
        // }

        // Resume the thread
        if let Err(e) = controller.send_command(thread_id, DeviceCommand::Start) {
            error!("Failed to resume thread: {}", e);
        }

        thread::sleep(Duration::from_millis(500));

        // Check the status
        match controller.get_status(thread_id) {
            Ok(status) => {
                info!("Thread status: {:?}", status);
                assert_eq!(status, ThreadStatus::Running);
            }
            Err(e) => {
                error!("Failed to get thread status: {}", e);
            }
        }

        // Set global log level to Trace
        // if let Err(e) = controller.set_global_log_level(LevelFilter::Trace) {
        //     error!("Failed to set global log level: {}", e);
        // }

        // // Stop the thread
        // if let Err(e) = controller.send_command(thread_id, DeviceCommand::Terminate) {
        //     error!("Failed to terminate thread {}: {}", thread_id, e);
        // }

        // Wait for the thread to terminate
        thread::sleep(Duration::from_millis(500));
    } else {
        warn!("No PoKeys devices found (USB or network)");
    }

    info!("Comprehensive example completed");
}
