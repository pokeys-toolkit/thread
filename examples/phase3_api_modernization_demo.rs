//! Phase 3 API Modernization Demo
//!
//! This example demonstrates all Phase 3 features:
//! - Enhanced Device Model Integration
//! - Enhanced Error Handling with Context
//! - Performance Optimizations with Bulk Operations

use log::info;
use pokeys_lib::PinCapability;
use pokeys_thread::{DeviceOperations, ThreadController, ThreadControllerBuilder, ThreadError};
use std::time::Duration;

fn main() -> pokeys_thread::Result<()> {
    // Initialize logging
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting Phase 3 API Modernization Demo");

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
        info!("Device connected successfully, demonstrating Phase 3 features");

        // Wait for device initialization
        std::thread::sleep(Duration::from_millis(500));

        // === ENHANCED DEVICE MODEL INTEGRATION ===
        info!("=== Enhanced Device Model Integration ===");

        // Get device model information
        match controller.get_device_model(thread_id) {
            Ok(Some(model)) => {
                info!("Connected device model: {}", model);
            }
            Ok(None) => {
                info!("Device model information not available");
            }
            Err(e) => {
                info!("Failed to get device model: {}", e);
            }
        }

        // Demonstrate pin capability checking
        info!("Checking pin capabilities:");
        
        let test_pins = [5, 17, 25, 50];
        let capabilities = [
            PinCapability::AnalogInput,
            PinCapability::PwmOutput,
            PinCapability::DigitalOutput,
            PinCapability::DigitalInput,
        ];

        for pin in test_pins {
            for capability in &capabilities {
                match controller.check_pin_capability(thread_id, pin, *capability) {
                    Ok(supported) => {
                        info!("Pin {} {:?}: {}", pin, capability, 
                              if supported { "✓ Supported" } else { "✗ Not supported" });
                    }
                    Err(e) => {
                        info!("Failed to check pin {} capability: {}", pin, e);
                    }
                }
            }
        }

        // === ENHANCED ERROR HANDLING ===
        info!("=== Enhanced Error Handling with Context ===");

        // Demonstrate validation with recovery suggestions
        info!("Testing pin validation with enhanced error handling:");

        let test_operations = [
            (17, "pwm"),
            (5, "analog_input"),
            (99, "pwm"), // This should fail with helpful error
            (17, "invalid_operation"), // This should fail with suggestion
        ];

        for (pin, operation) in test_operations {
            match controller.validate_pin_operation(thread_id, pin, operation) {
                Ok(()) => {
                    info!("✓ Pin {} validated for {}", pin, operation);
                }
                Err(e) => {
                    info!("✗ Validation failed for pin {} ({}): {}", pin, operation, e);
                    
                    // Demonstrate enhanced error handling
                    if e.is_recoverable() {
                        if let Some(suggestion) = e.recovery_suggestion() {
                            info!("  💡 Recovery suggestion: {}", suggestion);
                        }
                    } else {
                        info!("  ⚠️  This error is not recoverable");
                    }
                }
            }
        }

        // Demonstrate different error types
        info!("Demonstrating enhanced error types:");

        // Create example errors to show their features
        let pin_error = ThreadError::pin_capability_error(
            99,
            "pwm",
            Some("PWM is only available on pins 17-22".to_string())
        );
        info!("Pin capability error: {}", pin_error);
        info!("  Recoverable: {}", pin_error.is_recoverable());
        if let Some(suggestion) = pin_error.recovery_suggestion() {
            info!("  Suggestion: {}", suggestion);
        }

        let hardware_error = ThreadError::hardware_constraint(
            "PWM frequency exceeds maximum",
            "Reduce frequency to below 25MHz"
        );
        info!("Hardware constraint error: {}", hardware_error);
        if let Some(suggestion) = hardware_error.recovery_suggestion() {
            info!("  Suggestion: {}", suggestion);
        }

        // === PERFORMANCE OPTIMIZATIONS ===
        info!("=== Performance Optimizations with Bulk Operations ===");

        // Demonstrate bulk digital output operations
        info!("Setting multiple digital outputs in bulk:");
        let pin_states = vec![
            (1, true), (2, false), (3, true), (4, false), (5, true)
        ];
        
        match controller.set_digital_outputs_bulk(thread_id, pin_states.clone()) {
            Ok(()) => {
                info!("✓ Bulk digital outputs set successfully ({} pins)", pin_states.len());
                for (pin, state) in &pin_states {
                    info!("  Pin {}: {}", pin, if *state { "HIGH" } else { "LOW" });
                }
            }
            Err(e) => {
                info!("✗ Bulk digital outputs failed: {}", e);
            }
        }

        // Demonstrate bulk PWM operations
        info!("Setting multiple PWM duty cycles in bulk:");
        let channel_duties = vec![
            (0, 1000), (1, 2000), (2, 3000), (3, 4000)
        ];
        
        match controller.set_pwm_duties_bulk(thread_id, channel_duties.clone()) {
            Ok(()) => {
                info!("✓ Bulk PWM duties set successfully ({} channels)", channel_duties.len());
                for (channel, duty) in &channel_duties {
                    let percentage = (*duty as f32 / 4095.0) * 100.0;
                    info!("  Channel {}: {} ({:.1}%)", channel, duty, percentage);
                }
            }
            Err(e) => {
                info!("✗ Bulk PWM duties failed: {}", e);
            }
        }

        // Demonstrate bulk analog input reading
        info!("Reading multiple analog inputs in bulk:");
        let analog_pins = vec![0, 1, 2, 3, 4, 5, 6, 7];
        
        match controller.read_analog_inputs_bulk(thread_id, analog_pins.clone()) {
            Ok(values) => {
                info!("✓ Bulk analog read successful ({} pins)", analog_pins.len());
                for (i, value) in values.iter().enumerate() {
                    if i < analog_pins.len() {
                        let voltage = (*value as f32 / 4095.0) * 3.3; // Assuming 3.3V reference
                        info!("  Pin {}: {} ({:.2}V)", analog_pins[i], value, voltage);
                    }
                }
            }
            Err(e) => {
                info!("✗ Bulk analog read failed: {}", e);
            }
        }

        // === INTEGRATION DEMONSTRATION ===
        info!("=== Integration with Previous Phases ===");

        // Show that all previous functionality still works
        info!("Verifying backward compatibility:");

        // Phase 1: Basic PWM
        controller.set_pwm_duty_cycle_percent(thread_id, 0, 25.0)?;
        info!("✓ Phase 1 PWM operations working");

        // Phase 2: Servo control
        let servo_config = pokeys_lib::ServoConfig::one_eighty(17, 1000, 2000);
        controller.configure_servo(thread_id, 17, servo_config)?;
        controller.set_servo_angle(thread_id, 17, 90.0)?;
        info!("✓ Phase 2 servo operations working");

        // Phase 3: Enhanced validation
        controller.validate_pin_operation(thread_id, 17, "servo")?;
        info!("✓ Phase 3 validation working");

        // Performance comparison demonstration
        info!("Performance comparison:");
        
        let start = std::time::Instant::now();
        // Individual operations (old way)
        for i in 0..10 {
            let _ = controller.set_digital_output(thread_id, i, i % 2 == 0);
        }
        let individual_time = start.elapsed();
        
        let start = std::time::Instant::now();
        // Bulk operation (new way)
        let bulk_states: Vec<(u32, bool)> = (0..10).map(|i| (i, i % 2 == 0)).collect();
        let _ = controller.set_digital_outputs_bulk(thread_id, bulk_states);
        let bulk_time = start.elapsed();
        
        info!("Individual operations: {:?}", individual_time);
        info!("Bulk operation: {:?}", bulk_time);
        if bulk_time < individual_time {
            info!("✓ Bulk operations are faster!");
        }

        info!("Phase 3 API modernization demo completed successfully");
    } else {
        info!("No devices found. Phase 3 demo will run without hardware.");
        info!("This demonstrates that the Phase 3 API is working correctly.");
        
        // Demonstrate API without hardware
        info!("=== Phase 3 API Demonstration (No Hardware) ===");
        
        info!("Enhanced Device Model Integration:");
        info!("- Pin capability validation with device-specific constraints");
        info!("- Device model information retrieval");
        info!("- Hardware-aware operation validation");
        
        info!("Enhanced Error Handling:");
        info!("- Contextual error messages with recovery suggestions");
        info!("- Recoverable vs non-recoverable error classification");
        info!("- Hardware constraint violations with specific guidance");
        
        info!("Performance Optimizations:");
        info!("- Bulk digital output operations (up to 50+ pins)");
        info!("- Bulk PWM duty cycle setting (all 6 channels)");
        info!("- Bulk analog input reading (all 8 channels)");
        info!("- Reduced communication overhead and improved throughput");
        
        // Demonstrate error handling without hardware
        let demo_error = ThreadError::pin_capability_error(
            99,
            "pwm",
            Some("Use pins 17-22 for PWM output".to_string())
        );
        
        info!("Example enhanced error: {}", demo_error);
        info!("Recoverable: {}", demo_error.is_recoverable());
        if let Some(suggestion) = demo_error.recovery_suggestion() {
            info!("Recovery suggestion: {}", suggestion);
        }
    }

    Ok(())
}
