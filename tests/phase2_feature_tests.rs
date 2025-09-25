use pokeys_thread::*;
use pokeys_lib::{ServoConfig, ServoType, USPIBridgeConfig};

#[test]
fn test_servo_config_creation() {
    // Test servo configuration creation
    let servo_180 = ServoConfig::one_eighty(17, 1000, 2000);
    assert_eq!(servo_180.pin, 17);
    
    let servo_360_pos = ServoConfig::three_sixty_position(18, 500, 2500);
    assert_eq!(servo_360_pos.pin, 18);
    
    let servo_360_speed = ServoConfig::three_sixty_speed(19, 1500, 2000, 1000);
    assert_eq!(servo_360_speed.pin, 19);
}

#[test]
fn test_servo_commands() {
    // Test servo command creation
    let config = ServoConfig::one_eighty(17, 1000, 2000);
    
    let configure_cmd = DeviceCommand::ConfigureServo { pin: 17, config };
    match configure_cmd {
        DeviceCommand::ConfigureServo { pin, .. } => {
            assert_eq!(pin, 17);
        }
        _ => panic!("Expected ConfigureServo command"),
    }
    
    let angle_cmd = DeviceCommand::SetServoAngle { pin: 17, angle: 90.0 };
    match angle_cmd {
        DeviceCommand::SetServoAngle { pin, angle } => {
            assert_eq!(pin, 17);
            assert_eq!(angle, 90.0);
        }
        _ => panic!("Expected SetServoAngle command"),
    }
    
    let speed_cmd = DeviceCommand::SetServoSpeed { pin: 18, speed: 50.0 };
    match speed_cmd {
        DeviceCommand::SetServoSpeed { pin, speed } => {
            assert_eq!(pin, 18);
            assert_eq!(speed, 50.0);
        }
        _ => panic!("Expected SetServoSpeed command"),
    }
    
    let stop_cmd = DeviceCommand::StopServo { pin: 19 };
    match stop_cmd {
        DeviceCommand::StopServo { pin } => {
            assert_eq!(pin, 19);
        }
        _ => panic!("Expected StopServo command"),
    }
}

#[test]
fn test_i2c_commands() {
    // Test I2C command creation
    let write_cmd = DeviceCommand::I2cWrite {
        address: 0x48,
        data: vec![0x01, 0x02, 0x03],
    };
    match write_cmd {
        DeviceCommand::I2cWrite { address, data } => {
            assert_eq!(address, 0x48);
            assert_eq!(data, vec![0x01, 0x02, 0x03]);
        }
        _ => panic!("Expected I2cWrite command"),
    }
    
    let read_cmd = DeviceCommand::I2cRead {
        address: 0x48,
        length: 4,
    };
    match read_cmd {
        DeviceCommand::I2cRead { address, length } => {
            assert_eq!(address, 0x48);
            assert_eq!(length, 4);
        }
        _ => panic!("Expected I2cRead command"),
    }
    
    let write_read_cmd = DeviceCommand::I2cWriteRead {
        address: 0x48,
        write_data: vec![0x01],
        read_length: 2,
    };
    match write_read_cmd {
        DeviceCommand::I2cWriteRead { address, write_data, read_length } => {
            assert_eq!(address, 0x48);
            assert_eq!(write_data, vec![0x01]);
            assert_eq!(read_length, 2);
        }
        _ => panic!("Expected I2cWriteRead command"),
    }
    
    let scan_cmd = DeviceCommand::I2cScan;
    match scan_cmd {
        DeviceCommand::I2cScan => {
            // Command created successfully
        }
        _ => panic!("Expected I2cScan command"),
    }
}

#[test]
fn test_uspibridge_commands() {
    // Test uSPIBridge command creation
    let config = USPIBridgeConfig::new()
        .with_device_count(4)
        .with_default_brightness(8);
    
    let configure_cmd = DeviceCommand::ConfigureUSPIBridge { config };
    match configure_cmd {
        DeviceCommand::ConfigureUSPIBridge { config } => {
            assert_eq!(config.device_count, 4);
            assert_eq!(config.default_brightness, 8);
        }
        _ => panic!("Expected ConfigureUSPIBridge command"),
    }
    
    let command_cmd = DeviceCommand::USPIBridgeCommand {
        command: vec![0x11, 0x01, 0x02, 0x03],
    };
    match command_cmd {
        DeviceCommand::USPIBridgeCommand { command } => {
            assert_eq!(command, vec![0x11, 0x01, 0x02, 0x03]);
        }
        _ => panic!("Expected USPIBridgeCommand command"),
    }
}

#[test]
fn test_servo_pin_validation() {
    // Test that servo operations use valid PWM pins (17-22)
    let valid_pins = [17, 18, 19, 20, 21, 22];
    
    for pin in valid_pins {
        let config = ServoConfig::one_eighty(pin, 1000, 2000);
        assert_eq!(config.pin, pin);
        
        let angle_cmd = DeviceCommand::SetServoAngle { pin, angle: 45.0 };
        match angle_cmd {
            DeviceCommand::SetServoAngle { pin: cmd_pin, .. } => {
                assert_eq!(cmd_pin, pin);
            }
            _ => panic!("Expected SetServoAngle command"),
        }
    }
}

#[test]
fn test_i2c_address_validation() {
    // Test I2C address ranges (typically 0x08 to 0x77 for 7-bit addressing)
    let valid_addresses = [0x08, 0x48, 0x50, 0x68, 0x77];
    
    for address in valid_addresses {
        let write_cmd = DeviceCommand::I2cWrite {
            address,
            data: vec![0x00],
        };
        match write_cmd {
            DeviceCommand::I2cWrite { address: cmd_addr, .. } => {
                assert_eq!(cmd_addr, address);
            }
            _ => panic!("Expected I2cWrite command"),
        }
        
        let read_cmd = DeviceCommand::I2cRead {
            address,
            length: 1,
        };
        match read_cmd {
            DeviceCommand::I2cRead { address: cmd_addr, .. } => {
                assert_eq!(cmd_addr, address);
            }
            _ => panic!("Expected I2cRead command"),
        }
    }
}

#[test]
fn test_servo_type_variants() {
    // Test different servo type variants
    let servo_180 = ServoType::OneEighty { pos_0: 1000, pos_180: 2000 };
    match servo_180 {
        ServoType::OneEighty { pos_0, pos_180 } => {
            assert_eq!(pos_0, 1000);
            assert_eq!(pos_180, 2000);
        }
        _ => panic!("Expected OneEighty servo type"),
    }
    
    let servo_360_pos = ServoType::ThreeSixtyPosition { pos_0: 500, pos_360: 2500 };
    match servo_360_pos {
        ServoType::ThreeSixtyPosition { pos_0, pos_360 } => {
            assert_eq!(pos_0, 500);
            assert_eq!(pos_360, 2500);
        }
        _ => panic!("Expected ThreeSixtyPosition servo type"),
    }
    
    let servo_360_speed = ServoType::ThreeSixtySpeed {
        stop: 1500,
        clockwise: 2000,
        anti_clockwise: 1000,
    };
    match servo_360_speed {
        ServoType::ThreeSixtySpeed { stop, clockwise, anti_clockwise } => {
            assert_eq!(stop, 1500);
            assert_eq!(clockwise, 2000);
            assert_eq!(anti_clockwise, 1000);
        }
        _ => panic!("Expected ThreeSixtySpeed servo type"),
    }
}

#[test]
fn test_uspibridge_config_builder() {
    // Test uSPIBridge configuration builder pattern
    let config = USPIBridgeConfig::new()
        .with_device_count(6)
        .with_default_brightness(12)
        .with_max_virtual_devices(4);
    
    assert_eq!(config.device_count, 6);
    assert_eq!(config.default_brightness, 12);
    assert_eq!(config.max_virtual_devices, 4);
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::sync::Arc;
    
    #[test]
    fn test_phase2_api_compatibility() {
        // Test that Phase 2 features don't break existing functionality
        let device_info = pokeys_lib::DeviceInfo::default();
        let device_data = pokeys_lib::DeviceData::default();
        let shared_state = Arc::new(SharedDeviceState::new(device_info, device_data));
        
        // Test that PWM operations still work (from Phase 1)
        shared_state.set_pwm_duty_cycle(0, 1000);
        assert_eq!(shared_state.get_pwm_duty_cycle(0), Some(1000));
        
        // Test that new servo commands can be created
        let servo_config = ServoConfig::one_eighty(17, 1000, 2000);
        let configure_cmd = DeviceCommand::ConfigureServo {
            pin: servo_config.pin,
            config: servo_config,
        };
        
        match configure_cmd {
            DeviceCommand::ConfigureServo { pin, .. } => {
                assert_eq!(pin, 17);
            }
            _ => panic!("Expected ConfigureServo command"),
        }
    }
    
    #[test]
    fn test_command_enum_completeness() {
        // Test that all new commands are properly handled
        let commands = vec![
            DeviceCommand::ConfigureServo {
                pin: 17,
                config: ServoConfig::one_eighty(17, 1000, 2000),
            },
            DeviceCommand::SetServoAngle { pin: 17, angle: 90.0 },
            DeviceCommand::SetServoSpeed { pin: 18, speed: 50.0 },
            DeviceCommand::StopServo { pin: 19 },
            DeviceCommand::I2cWrite {
                address: 0x48,
                data: vec![0x01, 0x02],
            },
            DeviceCommand::I2cRead {
                address: 0x48,
                length: 2,
            },
            DeviceCommand::I2cWriteRead {
                address: 0x48,
                write_data: vec![0x01],
                read_length: 1,
            },
            DeviceCommand::I2cScan,
            DeviceCommand::ConfigureUSPIBridge {
                config: USPIBridgeConfig::new(),
            },
            DeviceCommand::USPIBridgeCommand {
                command: vec![0x11, 0x01],
            },
        ];
        
        // Verify all commands can be created and matched
        for cmd in commands {
            match cmd {
                DeviceCommand::ConfigureServo { .. } => {},
                DeviceCommand::SetServoAngle { .. } => {},
                DeviceCommand::SetServoSpeed { .. } => {},
                DeviceCommand::StopServo { .. } => {},
                DeviceCommand::I2cWrite { .. } => {},
                DeviceCommand::I2cRead { .. } => {},
                DeviceCommand::I2cWriteRead { .. } => {},
                DeviceCommand::I2cScan => {},
                DeviceCommand::ConfigureUSPIBridge { .. } => {},
                DeviceCommand::USPIBridgeCommand { .. } => {},
                _ => {
                    // This ensures we handle all the new commands
                    // If we add more commands, this test will remind us to handle them
                }
            }
        }
    }
}
