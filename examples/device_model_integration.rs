#![allow(clippy::uninlined_format_args)]
//! Device model integration example
//!
//! This example demonstrates how to work with device models in the threading system:
//! - Loading device models automatically
//! - Validating pin capabilities before operations
//! - Monitoring model changes
//! - Working with device-specific features

use log::{error, info, warn};
use pokeys_thread::{DeviceOperations, ThreadController, ThreadControllerBuilder};
use std::thread;
use std::time::Duration;

fn main() {
    // Initialize logging
    env_logger::init();

    info!("Starting device model integration example");

    // Create a thread controller
    let mut controller = ThreadControllerBuilder::new()
        .default_refresh_interval(100)
        .build();

    let mut thread_id = None;

    // Try to find and connect to a device
    match controller.discover_usb_devices() {
        Ok(devices) => {
            info!("Found {} USB devices", devices.len());

            if !devices.is_empty() {
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

    // If no USB device, try network devices
    if thread_id.is_none() {
        match controller.discover_network_devices(2000) {
            Ok(devices) => {
                info!("Found {} network devices", devices.len());

                if !devices.is_empty() {
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
        // Wait for the thread to initialize
        thread::sleep(Duration::from_millis(500));

        // Get device information
        match controller.get_state(thread_id) {
            Ok(state) => {
                info!("=== Device Information ===");
                info!("Serial Number: {}", state.device_data.serial_number);
                info!("Device Type: {}", state.device_data.device_type_name());
                info!(
                    "Firmware: {}.{}",
                    state.device_data.firmware_version_major,
                    state.device_data.firmware_version_minor
                );
                info!("Build Date: {}", state.device_data.build_date_string());

                info!("=== Device Capabilities ===");
                info!("Pin Count: {}", state.device_info.pin_count);
                info!("PWM Channels: {}", state.device_info.pwm_count);
                info!("Analog Inputs: {}", state.device_info.analog_inputs);
                info!("Encoders: {}", state.device_info.encoders_count);

                // Check if device model is loaded
                if let Some(model) = &state.model {
                    info!("=== Device Model Information ===");
                    info!("Model Name: {}", model.name);
                    info!("Pin Definitions: {}", model.pins.len());

                    // Show some pin capabilities
                    info!("=== Pin Capabilities (first 10 pins) ===");
                    for (pin_id, pin_def) in model.pins.iter().take(10) {
                        info!("Pin {}: Active: {}", pin_id, pin_def.active);
                        info!("  Capabilities: {:?}", pin_def.capabilities);
                    }

                    // Demonstrate safe pin operations using model validation
                    info!("=== Safe Pin Operations ===");

                    // Try to find a pin that supports digital output
                    let mut digital_output_pin = None;
                    for (pin_id, pin_def) in &model.pins {
                        if pin_def.capabilities.contains(&"digital_output".to_string()) {
                            digital_output_pin = Some(*pin_id as u32);
                            info!("Found digital output capable pin: {}", pin_id);
                            break;
                        }
                    }

                    if let Some(pin) = digital_output_pin {
                        info!("Setting digital output on pin {}", pin);
                        if let Err(e) = controller.set_digital_output(thread_id, pin, true) {
                            error!("Failed to set digital output: {}", e);
                        } else {
                            info!("Successfully set pin {} to HIGH", pin);

                            thread::sleep(Duration::from_millis(1000));

                            if let Err(e) = controller.set_digital_output(thread_id, pin, false) {
                                error!("Failed to clear digital output: {}", e);
                            } else {
                                info!("Successfully set pin {} to LOW", pin);
                            }
                        }
                    }

                    // Try to find a pin that supports analog input
                    let mut analog_input_pin = None;
                    for (pin_id, pin_def) in &model.pins {
                        if pin_def.capabilities.contains(&"analog_input".to_string()) {
                            analog_input_pin = Some(*pin_id as u32);
                            info!("Found analog input capable pin: {}", pin_id);
                            break;
                        }
                    }

                    if let Some(pin) = analog_input_pin {
                        match controller.get_analog_input(thread_id, pin) {
                            Ok(value) => {
                                let voltage = (value as f32 / 4095.0) * 5.0; // Assuming 5V reference
                                info!(
                                    "Analog input pin {}: {} (raw) / {:.3}V",
                                    pin, value, voltage
                                );
                            }
                            Err(e) => {
                                error!("Failed to read analog input: {}", e);
                            }
                        }
                    }

                    // Try to find a PWM capable pin
                    let mut pwm_pin = None;
                    for (pin_id, pin_def) in &model.pins {
                        if pin_def.capabilities.contains(&"pwm_output".to_string()) {
                            pwm_pin = Some(*pin_id);
                            info!("Found PWM capable pin: {}", pin_id);
                            break;
                        }
                    }

                    if let Some(_pin) = pwm_pin {
                        // Note: PWM operations typically use channel numbers, not pin numbers
                        if state.device_info.pwm_count > 0 {
                            info!("Setting PWM channel 0 to 25% duty cycle");
                            if let Err(e) =
                                controller.set_pwm_duty_cycle_percent(thread_id, 0, 25.0)
                            {
                                error!("Failed to set PWM duty cycle: {}", e);
                            } else {
                                info!("PWM set successfully");

                                thread::sleep(Duration::from_millis(2000));

                                info!("Setting PWM channel 0 to 0% duty cycle");
                                if let Err(e) =
                                    controller.set_pwm_duty_cycle_percent(thread_id, 0, 0.0)
                                {
                                    error!("Failed to clear PWM: {}", e);
                                } else {
                                    info!("PWM cleared successfully");
                                }
                            }
                        }
                    }
                } else {
                    warn!("No device model loaded - operations may not be validated");

                    // Demonstrate operations without model validation
                    info!("=== Basic Operations (without model validation) ===");

                    // Try basic digital output (pin 1 is commonly available)
                    info!("Attempting digital output on pin 1 (common pin)");
                    if let Err(e) = controller.set_digital_output(thread_id, 1, true) {
                        error!("Failed to set digital output: {}", e);
                    } else {
                        info!("Set pin 1 to HIGH");
                        thread::sleep(Duration::from_millis(1000));

                        if let Err(e) = controller.set_digital_output(thread_id, 1, false) {
                            error!("Failed to clear digital output: {}", e);
                        } else {
                            info!("Set pin 1 to LOW");
                        }
                    }
                }

                // Start model monitoring if available
                info!("=== Model Monitoring ===");
                match controller.start_model_monitoring(thread_id, None) {
                    Ok(_) => {
                        info!("Started model monitoring for thread {}", thread_id);

                        // Let it monitor for a few seconds
                        thread::sleep(Duration::from_millis(3000));

                        match controller.stop_model_monitoring(thread_id) {
                            Ok(_) => {
                                info!("Stopped model monitoring");
                            }
                            Err(e) => {
                                error!("Failed to stop model monitoring: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Model monitoring not available: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to get device state: {}", e);
            }
        }

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
        warn!("No PoKeys devices found");
        info!("Make sure a PoKeys device is connected via USB or available on the network");
    }

    info!("Device model integration example completed");
}
