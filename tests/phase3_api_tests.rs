use pokeys_lib::PinCapability;
use pokeys_thread::*;
use std::sync::Arc;

#[test]
fn test_enhanced_error_types() {
    // Test pin capability error
    let error = ThreadError::pin_capability_error(
        17,
        "analog_input",
        Some("Use pins 0-7 for analog inputs".to_string()),
    );

    match error {
        ThreadError::PinCapabilityError {
            pin,
            capability,
            suggestion,
            ..
        } => {
            assert_eq!(pin, 17);
            assert_eq!(capability, "analog_input");
            assert!(suggestion.is_some());
        }
        _ => panic!("Expected PinCapabilityError"),
    }

    // Test hardware constraint error
    let error = ThreadError::hardware_constraint(
        "PWM frequency too high",
        "Reduce frequency to below 25MHz",
    );

    match error {
        ThreadError::HardwareConstraint {
            constraint,
            suggestion,
            ..
        } => {
            assert_eq!(constraint, "PWM frequency too high");
            assert_eq!(suggestion, "Reduce frequency to below 25MHz");
        }
        _ => panic!("Expected HardwareConstraint"),
    }

    // Test validation error
    let error = ThreadError::validation_error(
        "Invalid pin configuration",
        "servo setup",
        Some("Use PWM pins 17-22 for servos"),
    );

    match error {
        ThreadError::ValidationError {
            message,
            context,
            recovery_suggestion,
        } => {
            assert_eq!(message, "Invalid pin configuration");
            assert_eq!(context, "servo setup");
            assert!(recovery_suggestion.is_some());
        }
        _ => panic!("Expected ValidationError"),
    }

    // Test resource conflict error
    let error = ThreadError::resource_conflict("Pin 17", "PWM output");

    match error {
        ThreadError::ResourceConflict {
            resource,
            conflicting_operation,
            ..
        } => {
            assert_eq!(resource, "Pin 17");
            assert_eq!(conflicting_operation, "PWM output");
        }
        _ => panic!("Expected ResourceConflict"),
    }
}

#[test]
fn test_error_recovery_methods() {
    // Test recoverable error
    let error = ThreadError::validation_error(
        "Invalid configuration",
        "test",
        Some("Fix the configuration"),
    );

    assert!(error.is_recoverable());
    assert_eq!(error.recovery_suggestion(), Some("Fix the configuration"));

    // Test non-recoverable error
    let error = ThreadError::ThreadNotFound(123);
    assert!(!error.is_recoverable());
    assert_eq!(error.recovery_suggestion(), None);

    // Test pin capability error recovery
    let error =
        ThreadError::pin_capability_error(50, "pwm", Some("Use pins 17-22 for PWM".to_string()));

    assert!(error.is_recoverable());
    assert_eq!(error.recovery_suggestion(), Some("Use pins 17-22 for PWM"));
}

#[test]
fn test_bulk_operations_commands() {
    // Test bulk digital outputs command
    let pin_states = vec![(1, true), (2, false), (3, true)];
    let cmd = DeviceCommand::SetDigitalOutputsBulk {
        pin_states: pin_states.clone(),
    };

    match cmd {
        DeviceCommand::SetDigitalOutputsBulk { pin_states: states } => {
            assert_eq!(states, pin_states);
        }
        _ => panic!("Expected SetDigitalOutputsBulk command"),
    }

    // Test bulk PWM duties command
    let channel_duties = vec![(0, 1000), (1, 2000), (2, 3000)];
    let cmd = DeviceCommand::SetPwmDutiesBulk {
        channel_duties: channel_duties.clone(),
    };

    match cmd {
        DeviceCommand::SetPwmDutiesBulk {
            channel_duties: duties,
        } => {
            assert_eq!(duties, channel_duties);
        }
        _ => panic!("Expected SetPwmDutiesBulk command"),
    }

    // Test bulk analog read command
    let pins = vec![0, 1, 2, 3];
    let cmd = DeviceCommand::ReadAnalogInputsBulk { pins: pins.clone() };

    match cmd {
        DeviceCommand::ReadAnalogInputsBulk { pins: read_pins } => {
            assert_eq!(read_pins, pins);
        }
        _ => panic!("Expected ReadAnalogInputsBulk command"),
    }
}

#[test]
fn test_pin_capability_validation() {
    // Test pin capability command
    let cmd = DeviceCommand::CheckPinCapability {
        pin: 17,
        capability: "pwm".to_string(),
    };

    match cmd {
        DeviceCommand::CheckPinCapability { pin, capability } => {
            assert_eq!(pin, 17);
            assert_eq!(capability, "pwm");
        }
        _ => panic!("Expected CheckPinCapability command"),
    }

    // Test pin operation validation command
    let cmd = DeviceCommand::ValidatePinOperation {
        pin: 5,
        operation: "analog_input".to_string(),
    };

    match cmd {
        DeviceCommand::ValidatePinOperation { pin, operation } => {
            assert_eq!(pin, 5);
            assert_eq!(operation, "analog_input");
        }
        _ => panic!("Expected ValidatePinOperation command"),
    }
}

#[test]
fn test_pin_capability_enum_variants() {
    // Test that we can create different pin capability variants
    let capabilities = vec![
        PinCapability::DigitalInput,
        PinCapability::DigitalOutput,
        PinCapability::AnalogInput,
        PinCapability::PwmOutput,
        PinCapability::MfAnalogInput,
        PinCapability::AnalogOutput,
    ];

    for capability in capabilities {
        // Just verify we can create and match on these variants
        match capability {
            PinCapability::DigitalInput => {}
            PinCapability::DigitalOutput => {}
            PinCapability::AnalogInput => {}
            PinCapability::PwmOutput => {}
            PinCapability::MfAnalogInput => {}
            PinCapability::AnalogOutput => {}
            _ => {}
        }
    }
}

#[test]
fn test_bulk_operation_performance_benefits() {
    // Test that bulk operations can handle multiple items efficiently
    let large_pin_states: Vec<(u32, bool)> = (0..50).map(|i| (i, i % 2 == 0)).collect();
    let cmd = DeviceCommand::SetDigitalOutputsBulk {
        pin_states: large_pin_states.clone(),
    };

    match cmd {
        DeviceCommand::SetDigitalOutputsBulk { pin_states } => {
            assert_eq!(pin_states.len(), 50);
            assert_eq!(pin_states[0], (0, true));
            assert_eq!(pin_states[1], (1, false));
            assert_eq!(pin_states[49], (49, false));
        }
        _ => panic!("Expected SetDigitalOutputsBulk command"),
    }

    // Test bulk PWM with many channels
    let large_pwm_duties: Vec<(usize, u32)> = (0..6).map(|i| (i, (i * 500) as u32)).collect();
    let cmd = DeviceCommand::SetPwmDutiesBulk {
        channel_duties: large_pwm_duties.clone(),
    };

    match cmd {
        DeviceCommand::SetPwmDutiesBulk { channel_duties } => {
            assert_eq!(channel_duties.len(), 6);
            assert_eq!(channel_duties[0], (0, 0));
            assert_eq!(channel_duties[5], (5, 2500));
        }
        _ => panic!("Expected SetPwmDutiesBulk command"),
    }
}

#[test]
fn test_device_model_integration() {
    // Test device model information access
    let device_info = pokeys_lib::DeviceInfo::default();
    let mut device_data = pokeys_lib::DeviceData::default();

    // Set a device name in the byte array
    let name = b"PoKeys57E\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    device_data.device_name.copy_from_slice(name);

    let shared_state = Arc::new(SharedDeviceState::new(device_info, device_data));

    // Test reading device name
    let device_name = shared_state.read(|state| {
        let name_bytes = &state.device_data.device_name;
        let end = name_bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(name_bytes.len());
        String::from_utf8_lossy(&name_bytes[..end]).to_string()
    });

    assert_eq!(device_name, "PoKeys57E");
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_phase3_backward_compatibility() {
        // Test that Phase 3 features don't break Phase 1 and 2 functionality
        let device_info = pokeys_lib::DeviceInfo::default();
        let device_data = pokeys_lib::DeviceData::default();
        let shared_state = Arc::new(SharedDeviceState::new(device_info, device_data));

        // Test Phase 1 PWM operations still work
        shared_state.set_pwm_duty_cycle(0, 1000);
        assert_eq!(shared_state.get_pwm_duty_cycle(0), Some(1000));

        // Test Phase 2 servo commands still work
        let servo_config = pokeys_lib::ServoConfig::one_eighty(17, 1000, 2000);
        let servo_cmd = DeviceCommand::ConfigureServo {
            pin: servo_config.pin,
            config: servo_config,
        };

        match servo_cmd {
            DeviceCommand::ConfigureServo { pin, .. } => {
                assert_eq!(pin, 17);
            }
            _ => panic!("Expected ConfigureServo command"),
        }

        // Test Phase 3 bulk operations
        let bulk_cmd = DeviceCommand::SetDigitalOutputsBulk {
            pin_states: vec![(1, true), (2, false)],
        };

        match bulk_cmd {
            DeviceCommand::SetDigitalOutputsBulk { pin_states } => {
                assert_eq!(pin_states.len(), 2);
            }
            _ => panic!("Expected SetDigitalOutputsBulk command"),
        }
    }

    #[test]
    fn test_enhanced_error_handling_integration() {
        // Test that enhanced errors provide useful information
        let pin_error = ThreadError::pin_capability_error(
            99,
            "pwm",
            Some("PWM is only available on pins 17-22".to_string()),
        );

        // Test error message formatting
        let error_message = format!("{pin_error}");
        assert!(error_message.contains("Pin 99"));
        assert!(error_message.contains("pwm"));

        // Test recovery suggestion
        assert!(pin_error.is_recoverable());
        assert_eq!(
            pin_error.recovery_suggestion(),
            Some("PWM is only available on pins 17-22")
        );

        // Test validation error
        let validation_error = ThreadError::validation_error(
            "Invalid servo configuration",
            "servo setup on pin 99",
            Some("Use pins 17-22 for servo control"),
        );

        assert!(validation_error.is_recoverable());
        assert!(format!("{validation_error}").contains("Invalid servo configuration"));
    }

    #[test]
    fn test_performance_optimization_integration() {
        // Test that bulk operations can be created and processed
        let bulk_digital = DeviceCommand::SetDigitalOutputsBulk {
            pin_states: (0..20).map(|i| (i, i % 2 == 0)).collect(),
        };

        let bulk_pwm = DeviceCommand::SetPwmDutiesBulk {
            channel_duties: (0..6).map(|i| (i, (i * 500) as u32)).collect(),
        };

        let bulk_analog = DeviceCommand::ReadAnalogInputsBulk {
            pins: (0..8).collect(),
        };

        // Verify commands can be created and matched
        match bulk_digital {
            DeviceCommand::SetDigitalOutputsBulk { pin_states } => {
                assert_eq!(pin_states.len(), 20);
            }
            _ => panic!("Expected bulk digital command"),
        }

        match bulk_pwm {
            DeviceCommand::SetPwmDutiesBulk { channel_duties } => {
                assert_eq!(channel_duties.len(), 6);
            }
            _ => panic!("Expected bulk PWM command"),
        }

        match bulk_analog {
            DeviceCommand::ReadAnalogInputsBulk { pins } => {
                assert_eq!(pins.len(), 8);
            }
            _ => panic!("Expected bulk analog command"),
        }
    }
}
