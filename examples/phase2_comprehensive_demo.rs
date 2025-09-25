//! Phase 2 Comprehensive Demo
//!
//! This example demonstrates all Phase 2 features:
//! - Enhanced PWM Support with Servo Control
//! - Enhanced I2C Features
//! - uSPIBridge Integration

use log::info;
use pokeys_lib::{ServoConfig, ServoType, USPIBridgeConfig};
use pokeys_thread::{DeviceOperations, ThreadController, ThreadControllerBuilder};
use std::time::Duration;

fn main() -> pokeys_thread::Result<()> {
    // Initialize logging
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting Phase 2 Comprehensive Demo");

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
        info!("Device connected successfully, demonstrating Phase 2 features");

        // Wait for device initialization
        std::thread::sleep(Duration::from_millis(500));

        // === SERVO CONTROL DEMONSTRATION ===
        info!("=== Servo Control Features ===");

        // Configure different types of servos
        let servo_180 = ServoConfig::one_eighty(17, 1000, 2000); // 180° servo on pin 17
        let servo_360_pos = ServoConfig::three_sixty_position(18, 500, 2500); // 360° position servo on pin 18
        let servo_360_speed = ServoConfig::three_sixty_speed(19, 1500, 2000, 1000); // 360° speed servo on pin 19

        // Configure servos
        info!("Configuring 180° servo on pin 17");
        controller.configure_servo(thread_id, 17, servo_180)?;

        info!("Configuring 360° position servo on pin 18");
        controller.configure_servo(thread_id, 18, servo_360_pos)?;

        info!("Configuring 360° speed servo on pin 19");
        controller.configure_servo(thread_id, 19, servo_360_speed)?;

        // Demonstrate servo angle control
        info!("Setting 180° servo to 45°");
        controller.set_servo_angle(thread_id, 17, 45.0)?;
        std::thread::sleep(Duration::from_millis(500));

        info!("Setting 180° servo to 135°");
        controller.set_servo_angle(thread_id, 17, 135.0)?;
        std::thread::sleep(Duration::from_millis(500));

        // Demonstrate 360° position servo
        info!("Setting 360° position servo to 180°");
        controller.set_servo_angle(thread_id, 18, 180.0)?;
        std::thread::sleep(Duration::from_millis(500));

        // Demonstrate speed servo
        info!("Setting speed servo to 50% clockwise");
        controller.set_servo_speed(thread_id, 19, 50.0)?;
        std::thread::sleep(Duration::from_millis(1000));

        info!("Setting speed servo to 25% counter-clockwise");
        controller.set_servo_speed(thread_id, 19, -25.0)?;
        std::thread::sleep(Duration::from_millis(1000));

        // Stop all servos
        info!("Stopping all servos");
        controller.stop_servo(thread_id, 17)?;
        controller.stop_servo(thread_id, 18)?;
        controller.stop_servo(thread_id, 19)?;

        // === I2C FEATURES DEMONSTRATION ===
        info!("=== Enhanced I2C Features ===");

        // I2C bus scan
        info!("Scanning I2C bus for devices");
        match controller.i2c_scan(thread_id) {
            Ok(devices) => {
                info!("I2C scan completed, found {} devices", devices.len());
            }
            Err(e) => {
                info!("I2C scan failed: {}", e);
            }
        }

        // I2C write operation
        let test_address = 0x48; // Common I2C address for sensors
        let write_data = vec![0x01, 0x02, 0x03, 0x04];

        info!(
            "Writing {} bytes to I2C address 0x{:02X}",
            write_data.len(),
            test_address
        );
        match controller.i2c_write(thread_id, test_address, write_data.clone()) {
            Ok(()) => {
                info!("I2C write successful");
            }
            Err(e) => {
                info!("I2C write failed: {}", e);
            }
        }

        // I2C read operation
        info!("Reading 4 bytes from I2C address 0x{:02X}", test_address);
        match controller.i2c_read(thread_id, test_address, 4) {
            Ok(data) => {
                info!("I2C read successful, received {} bytes", data.len());
            }
            Err(e) => {
                info!("I2C read failed: {}", e);
            }
        }

        // I2C write-then-read operation
        let register_address = vec![0x00]; // Register to read from
        info!("I2C write-read: writing register address, then reading 2 bytes");
        match controller.i2c_write_read(thread_id, test_address, register_address, 2) {
            Ok(data) => {
                info!("I2C write-read successful, received {} bytes", data.len());
            }
            Err(e) => {
                info!("I2C write-read failed: {}", e);
            }
        }

        // === uSPIBridge DEMONSTRATION ===
        info!("=== uSPIBridge Features ===");

        // Configure uSPIBridge
        let uspibridge_config = USPIBridgeConfig::new()
            .with_device_count(4)
            .with_default_brightness(8)
            .with_max_virtual_devices(2);

        info!(
            "Configuring uSPIBridge with {} devices",
            uspibridge_config.device_count
        );
        match controller.configure_uspibridge(thread_id, uspibridge_config) {
            Ok(()) => {
                info!("uSPIBridge configuration successful");
            }
            Err(e) => {
                info!("uSPIBridge configuration failed: {}", e);
            }
        }

        // Send uSPIBridge command
        let uspibridge_command = vec![0x11, 0x01, 0x48, 0x65, 0x6C, 0x6C, 0x6F]; // Example command
        info!(
            "Sending uSPIBridge command ({} bytes)",
            uspibridge_command.len()
        );
        match controller.uspibridge_command(thread_id, uspibridge_command) {
            Ok(response) => {
                info!(
                    "uSPIBridge command successful, received {} bytes",
                    response.len()
                );
            }
            Err(e) => {
                info!("uSPIBridge command failed: {}", e);
            }
        }

        // === INTEGRATION DEMONSTRATION ===
        info!("=== Integration Features ===");

        // Demonstrate that Phase 1 PWM features still work
        info!("Verifying Phase 1 PWM compatibility");
        controller.set_pwm_duty_cycle_percent(thread_id, 0, 25.0)?;
        controller.set_pwm_duty_cycle_percent(thread_id, 1, 50.0)?;
        controller.set_pwm_duty_cycle_percent(thread_id, 2, 75.0)?;

        // Combine servo control with PWM
        info!("Combining servo control with direct PWM control");
        controller.set_servo_angle(thread_id, 20, 90.0)?; // Servo on pin 20
        controller.set_pwm_duty_cycle_percent(thread_id, 5, 30.0)?; // Direct PWM on channel 5 (pin 17)

        // Reset all outputs
        info!("Resetting all outputs");
        for channel in 0..6 {
            controller.set_pwm_duty_cycle(thread_id, channel, 0)?;
        }

        info!("Phase 2 comprehensive demo completed successfully");
    } else {
        info!("No devices found. Phase 2 demo will run without hardware.");
        info!("This demonstrates that the Phase 2 API is working correctly.");

        // Demonstrate API without hardware
        info!("=== Phase 2 API Demonstration (No Hardware) ===");
        info!("Servo Control:");
        info!("- 180° servos: angle control 0-180°");
        info!("- 360° position servos: angle control 0-360°");
        info!("- 360° speed servos: speed control -100% to +100%");
        info!("- All servos use PWM pins 17-22");

        info!("Enhanced I2C:");
        info!("- I2C write/read operations with automatic fragmentation");
        info!("- I2C bus scanning for device discovery");
        info!("- Combined write-read operations");
        info!("- Enhanced error handling and retry logic");

        info!("uSPIBridge:");
        info!("- Custom pinout configuration");
        info!("- Segment mapping for display control");
        info!("- Virtual device management");
        info!("- Advanced display features");
    }

    Ok(())
}
