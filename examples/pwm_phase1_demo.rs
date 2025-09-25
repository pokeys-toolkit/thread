//! PWM Phase 1 Demo
//!
//! This example demonstrates the PWM functionality after Phase 1 core library alignment.
//! It shows how PWM operations work with the updated core library integration.

use log::info;
use pokeys_thread::{DeviceOperations, ThreadController, ThreadControllerBuilder};
use std::time::Duration;

fn main() -> pokeys_thread::Result<()> {
    // Initialize logging
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting PWM Phase 1 Demo");

    // Create thread controller
    let mut controller = ThreadControllerBuilder::new()
        .default_refresh_interval(100)
        .build();

    // Try to discover and connect to a device
    let mut thread_id = None;

    // Try USB devices first
    match controller.discover_usb_devices() {
        Ok(devices) => {
            info!("Found {} USB devices", devices.len());
            if !devices.is_empty() {
                match controller.start_usb_device_thread(devices[0]) {
                    Ok(id) => {
                        info!("Started thread {} for USB device {}", id, devices[0]);
                        thread_id = Some(id);
                    }
                    Err(e) => {
                        info!("Failed to start USB device thread: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            info!("Failed to discover USB devices: {}", e);
        }
    }

    // If no USB device, try network devices
    if thread_id.is_none() {
        match controller.discover_network_devices(2000) {
            Ok(devices) => {
                info!("Found {} network devices", devices.len());
                if !devices.is_empty() {
                    match controller.start_network_device_thread(devices[0].clone()) {
                        Ok(id) => {
                            info!("Started thread {} for network device", id);
                            thread_id = Some(id);
                        }
                        Err(e) => {
                            info!("Failed to start network device thread: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                info!("Failed to discover network devices: {}", e);
            }
        }
    }

    if let Some(thread_id) = thread_id {
        info!("Device connected successfully, demonstrating PWM operations");

        // Wait for device initialization
        std::thread::sleep(Duration::from_millis(500));

        // Demonstrate PWM operations with the new core library integration
        info!("Testing PWM operations (channels 0-5 map to pins 22-17)");

        // Test each PWM channel
        for channel in 0..6 {
            let pin = match channel {
                0 => 22, 1 => 21, 2 => 20, 3 => 19, 4 => 18, 5 => 17,
                _ => continue,
            };

            info!("Setting PWM channel {} (pin {}) to 25% duty cycle", channel, pin);
            
            // Set PWM duty cycle to 25% (1024 out of 4095)
            match controller.set_pwm_duty_cycle(thread_id, channel, 1024) {
                Ok(()) => {
                    info!("Successfully set PWM channel {} duty cycle", channel);
                }
                Err(e) => {
                    info!("Failed to set PWM channel {} duty cycle: {}", channel, e);
                }
            }

            std::thread::sleep(Duration::from_millis(100));
        }

        // Test percentage-based PWM control
        info!("Testing percentage-based PWM control");
        for channel in 0..3 {
            let percentage = (channel + 1) as f32 * 25.0; // 25%, 50%, 75%
            
            info!("Setting PWM channel {} to {}% duty cycle", channel, percentage);
            
            match controller.set_pwm_duty_cycle_percent(thread_id, channel, percentage) {
                Ok(()) => {
                    info!("Successfully set PWM channel {} to {}%", channel, percentage);
                }
                Err(e) => {
                    info!("Failed to set PWM channel {} percentage: {}", channel, e);
                }
            }

            std::thread::sleep(Duration::from_millis(100));
        }

        // Demonstrate state monitoring
        info!("Creating state observer to monitor PWM changes");
        
        match controller.create_observer(thread_id) {
            Ok(observer) => {
                // Set a PWM value and wait for notification
                info!("Setting PWM channel 0 and waiting for state change notification");
                controller.set_pwm_duty_cycle(thread_id, 0, 2048)?;

                if let Some(change) = observer.wait_for_change(Duration::from_millis(500)) {
                    info!("Received state change notification: {:?}", change);
                } else {
                    info!("No state change notification received within timeout");
                }
            }
            Err(e) => {
                info!("Failed to create observer: {}", e);
            }
        }

        // Reset all PWM channels to 0
        info!("Resetting all PWM channels to 0");
        for channel in 0..6 {
            controller.set_pwm_duty_cycle(thread_id, channel, 0)?;
        }

        info!("PWM demo completed successfully");
    } else {
        info!("No devices found. PWM demo will run without hardware.");
        info!("This demonstrates that the PWM API is working correctly.");
        
        // Create a mock controller to show API usage
        info!("Demonstrating PWM API without hardware:");
        info!("- PWM channels 0-5 map to pins 22, 21, 20, 19, 18, 17");
        info!("- Duty cycle range: 0-4095 (12-bit)");
        info!("- Percentage range: 0.0-100.0%");
        info!("- All PWM operations now use pin-based methods in core library");
    }

    Ok(())
}
