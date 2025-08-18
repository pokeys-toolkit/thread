#![allow(clippy::uninlined_format_args)]
//! Example of using device operations

use log::{error, info};
use pokeys_thread::{DeviceOperations, ThreadController, ThreadControllerBuilder};
use std::thread;
use std::time::Duration;

fn main() {
    // Initialize logging
    env_logger::init();

    info!("Starting device operations example");

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

    // If we have a device thread, perform operations
    if let Some(thread_id) = thread_id {
        // Wait a bit for the thread to start
        thread::sleep(Duration::from_millis(100));

        // Get the device state
        match controller.get_state(thread_id) {
            Ok(state) => {
                info!(
                    "Device: Serial {}, FW {}.{}",
                    state.device_data.serial_number,
                    state.device_data.firmware_version_major,
                    state.device_data.firmware_version_minor
                );

                // Set a digital output
                if let Err(e) = controller.set_digital_output(thread_id, 1, true) {
                    error!("Failed to set digital output: {}", e);
                } else {
                    info!("Set digital output pin 1 to HIGH");
                }

                // Wait a bit
                thread::sleep(Duration::from_millis(1000));

                // Set the digital output back to low
                if let Err(e) = controller.set_digital_output(thread_id, 1, false) {
                    error!("Failed to set digital output: {}", e);
                } else {
                    info!("Set digital output pin 1 to LOW");
                }

                // Read a digital input
                match controller.get_digital_input(thread_id, 2) {
                    Ok(value) => {
                        info!(
                            "Digital input pin 2: {}",
                            if value { "HIGH" } else { "LOW" }
                        );
                    }
                    Err(e) => {
                        error!("Failed to read digital input: {}", e);
                    }
                }

                // Read an analog input
                match controller.get_analog_input(thread_id, 1) {
                    Ok(value) => {
                        let voltage = (value as f32 / 4095.0) * 5.0; // Assuming 5V reference
                        info!("Analog input pin 1: {} (raw) / {:.3}V", value, voltage);
                    }
                    Err(e) => {
                        error!("Failed to read analog input: {}", e);
                    }
                }

                // Set a PWM duty cycle
                if let Err(e) = controller.set_pwm_duty_cycle_percent(thread_id, 0, 50.0) {
                    error!("Failed to set PWM duty cycle: {}", e);
                } else {
                    info!("Set PWM channel 0 to 50% duty cycle");
                }

                // Wait a bit
                thread::sleep(Duration::from_millis(1000));

                // Set the PWM duty cycle to 0
                if let Err(e) = controller.set_pwm_duty_cycle_percent(thread_id, 0, 0.0) {
                    error!("Failed to set PWM duty cycle: {}", e);
                } else {
                    info!("Set PWM channel 0 to 0% duty cycle");
                }

                // Configure an encoder
                if let Err(e) = controller.configure_encoder(thread_id, 0, 1, 2, true, true) {
                    error!("Failed to configure encoder: {}", e);
                } else {
                    info!("Configured encoder 0 on pins 1 and 2");
                }

                // Read an encoder value
                match controller.get_encoder_value(thread_id, 0) {
                    Ok(value) => {
                        info!("Encoder 0 value: {}", value);
                    }
                    Err(e) => {
                        error!("Failed to read encoder value: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to get device state: {}", e);
            }
        }

        // Stop the thread
        if let Err(e) = controller.send_command(thread_id, pokeys_thread::DeviceCommand::Terminate)
        {
            error!("Failed to stop thread: {}", e);
        }
    } else {
        info!("No PoKeys devices found (USB or network)");
    }

    // Stop all threads
    if let Err(e) = controller.stop_all() {
        error!("Failed to stop all threads: {}", e);
    }
}
