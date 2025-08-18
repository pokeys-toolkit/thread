#![allow(clippy::uninlined_format_args)]
//! Digital and analog output tracking example
//!
//! This example demonstrates the new StateChangeType::DigitalOutput and
//! StateChangeType::AnalogOutput functionality that tracks changes to output pins.
//! It shows how to:
//! - Monitor digital output state changes
//! - Monitor analog output state changes
//! - Track output changes from both direct commands and device synchronization

use log::{error, info};
use pokeys_lib::PinFunction;
use pokeys_thread::{DeviceOperations, StateChangeType, ThreadController, ThreadControllerBuilder};
use std::thread;
use std::time::Duration;

fn main() {
    // Initialize logging
    env_logger::init();

    info!("Starting output tracking example");

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
        thread::sleep(Duration::from_millis(500));

        // Get device information
        match controller.get_state(thread_id) {
            Ok(state) => {
                info!("=== Device Information ===");
                info!("Serial Number: {}", state.device_data.serial_number);
                info!("Device Type: {}", state.device_data.device_type_name());
                info!("Pin Count: {}", state.device_info.pin_count);
            }
            Err(e) => {
                error!("Failed to get device state: {}", e);
                return;
            }
        }

        // Create a state observer to monitor output changes
        match controller.create_observer(thread_id) {
            Ok(observer) => {
                info!("=== Starting Output Tracking ===");
                info!("Created state observer for output tracking");

                // Start a thread to monitor state changes
                let monitor_thread = thread::spawn(move || {
                    info!("Output monitor thread started");

                    // Monitor for 30 seconds
                    let start_time = std::time::Instant::now();
                    while start_time.elapsed() < Duration::from_secs(30) {
                        if let Some(change_type) =
                            observer.wait_for_change(Duration::from_millis(100))
                        {
                            match change_type {
                                StateChangeType::DigitalOutput { pin, value } => {
                                    info!(
                                        "🔌 Digital output pin {} changed to {}",
                                        pin,
                                        if value { "HIGH" } else { "LOW" }
                                    );
                                }
                                StateChangeType::AnalogOutput { pin, value } => {
                                    let voltage = (value as f32 / 4095.0) * 5.0; // Assuming 5V reference
                                    info!(
                                        "📊 Analog output pin {} changed to {} (raw) / {:.3}V",
                                        pin, value, voltage
                                    );
                                }
                                StateChangeType::DigitalInput { pin, value } => {
                                    info!(
                                        "📥 Digital input pin {} changed to {}",
                                        pin,
                                        if value { "HIGH" } else { "LOW" }
                                    );
                                }
                                StateChangeType::AnalogInput { pin, value } => {
                                    let voltage = (value as f32 / 4095.0) * 5.0;
                                    info!(
                                        "📈 Analog input pin {} changed to {} (raw) / {:.3}V",
                                        pin, value, voltage
                                    );
                                }
                                StateChangeType::PwmDutyCycle { channel, duty } => {
                                    let percentage = (duty as f32 / 4095.0) * 100.0;
                                    info!(
                                        "⚡ PWM channel {} duty changed to {} ({:.1}%)",
                                        channel, duty, percentage
                                    );
                                }
                                StateChangeType::EncoderValue { index, value } => {
                                    info!("🔄 Encoder {} value changed to {}", index, value);
                                }
                                StateChangeType::ThreadStatus { status } => {
                                    info!("🔧 Thread status changed to {:?}", status);
                                }
                                StateChangeType::Error { message } => {
                                    if let Some(msg) = message {
                                        info!("❌ Error occurred: {}", msg);
                                    } else {
                                        info!("✅ Error cleared");
                                    }
                                }
                                StateChangeType::FullUpdate => {
                                    info!("🔄 Full state update received");
                                }
                                StateChangeType::CustomValue { key, value } => {
                                    info!("🏷️  Custom value {} changed to {}", key, value);
                                }
                            }
                        }
                    }

                    info!("Output monitor thread finished");
                });

                // Demonstrate digital output tracking
                info!("=== Digital Output Demonstration ===");

                // First configure pins as digital outputs
                info!("Configuring pins 1-5 as digital outputs");
                for pin in 1..=5 {
                    if let Err(e) =
                        controller.set_pin_function(thread_id, pin, PinFunction::DigitalOutput)
                    {
                        error!("Failed to configure pin {} as digital output: {}", pin, e);
                    } else {
                        info!("Successfully configured pin {} as digital output", pin);
                    }
                }

                // Wait for configuration to take effect
                thread::sleep(Duration::from_millis(500));

                // Test digital outputs on pins 1-5
                for pin in 1..=5 {
                    info!("Setting digital output pin {} to HIGH", pin);
                    if let Err(e) = controller.set_digital_output(thread_id, pin, true) {
                        error!("Failed to set digital output pin {}: {}", pin, e);
                    }
                    thread::sleep(Duration::from_millis(500));

                    info!("Setting digital output pin {} to LOW", pin);
                    if let Err(e) = controller.set_digital_output(thread_id, pin, false) {
                        error!("Failed to clear digital output pin {}: {}", pin, e);
                    }
                    thread::sleep(Duration::from_millis(500));
                }

                // Demonstrate analog output tracking (if supported)
                info!("=== Analog Output Demonstration ===");

                // Configure pins as analog outputs
                info!("Configuring pins 1-3 as analog outputs");
                for pin in 1..=3 {
                    if let Err(e) =
                        controller.set_pin_function(thread_id, pin, PinFunction::AnalogOutput)
                    {
                        error!("Failed to configure pin {} as analog output: {}", pin, e);
                    } else {
                        info!("Successfully configured pin {} as analog output", pin);
                    }
                }

                // Wait for configuration to take effect
                thread::sleep(Duration::from_millis(500));

                // Test analog outputs on pins 1-3
                for pin in 1..=3 {
                    for &value in &[1024, 2048, 3072, 4095, 0] {
                        let voltage = (value as f32 / 4095.0) * 5.0;
                        info!(
                            "Setting analog output pin {} to {} ({:.3}V)",
                            pin, value, voltage
                        );
                        if let Err(e) = controller.set_analog_output(thread_id, pin, value) {
                            error!("Failed to set analog output pin {}: {}", pin, e);
                        }
                        thread::sleep(Duration::from_millis(300));
                    }
                }

                // Demonstrate PWM output tracking
                info!("=== PWM Output Demonstration ===");

                // Test PWM channels 0-2
                for channel in 0..3 {
                    for &duty_percent in &[25.0, 50.0, 75.0, 100.0, 0.0] {
                        info!("Setting PWM channel {} to {}%", channel, duty_percent);
                        if let Err(e) =
                            controller.set_pwm_duty_cycle_percent(thread_id, channel, duty_percent)
                        {
                            error!("Failed to set PWM channel {}: {}", channel, e);
                        }
                        thread::sleep(Duration::from_millis(400));
                    }
                }

                // Demonstrate rapid output changes
                info!("=== Rapid Output Changes ===");
                info!("Rapidly toggling pin 1 to demonstrate change tracking");

                // Ensure pin 1 is configured as digital output (should already be done above)
                if let Err(e) =
                    controller.set_pin_function(thread_id, 1, PinFunction::DigitalOutput)
                {
                    error!("Failed to configure pin 1 as digital output: {}", e);
                }

                for i in 0..10 {
                    let value = i % 2 == 0;
                    if let Err(e) = controller.set_digital_output(thread_id, 1, value) {
                        error!("Failed to toggle digital output: {}", e);
                    }
                    thread::sleep(Duration::from_millis(100));
                }

                // Wait for the monitor thread to finish
                info!("Waiting for monitor thread to complete...");
                monitor_thread.join().unwrap();

                info!("=== Output Tracking Summary ===");
                info!("This example demonstrated:");
                info!("• Digital output state change tracking");
                info!("• Analog output state change tracking");
                info!("• PWM duty cycle change tracking");
                info!("• Real-time monitoring of all output changes");
                info!("• Integration with the state observer system");
            }
            Err(e) => {
                error!("Failed to create state observer: {}", e);
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
        info!("No PoKeys devices found (USB or network)");
        info!("Please connect a PoKeys device and try again");
    }

    info!("Output tracking example completed");
}
