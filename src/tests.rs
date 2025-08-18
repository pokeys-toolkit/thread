//! Unit tests for the pokeys-thread crate

#[cfg(test)]
use crate::commands::DeviceCommand;
use crate::error::ThreadError;
use crate::logging::{Logger, SimpleLogger};
use crate::state::{DeviceState, SharedDeviceState, StateChangeType, ThreadStatus};
use log::LevelFilter;
use pokeys_lib::{DeviceData, DeviceInfo};
use std::sync::Arc;
use std::time::Duration;

// Mock logger for testing
struct MockLogger {
    level: LevelFilter,
}

impl MockLogger {
    fn new(level: LevelFilter) -> Self {
        Self { level }
    }
}

impl Logger for MockLogger {
    fn log(&self, _level: log::Level, _target: &str, _message: &str) {
        // Do nothing in tests
    }

    fn set_level(&mut self, level: LevelFilter) {
        self.level = level;
    }

    fn level(&self) -> LevelFilter {
        self.level
    }
}

#[test]
fn test_device_state() {
    // Create a minimal DeviceInfo for testing
    let device_info = DeviceInfo {
        pin_count: 55,
        pwm_count: 6,
        basic_encoder_count: 25,
        encoders_count: 25,
        fast_encoders: 0,
        ultra_fast_encoders: 0,
        pwm_internal_frequency: 10000,
        analog_inputs: 7,
        key_mapping: 1,
        triggered_key_mapping: 1,
        key_repeat_delay: 0,
        digital_counters: 1,
        joystick_button_axis_mapping: 1,
        joystick_analog_to_digital_mapping: 1,
        macros: 1,
        matrix_keyboard: 1,
        matrix_keyboard_triggered_mapping: 1,
        lcd: 1,
        matrix_led: 1,
        connection_signal: 1,
        po_ext_bus: 1,
        po_net: 1,
        analog_filtering: 1,
        init_outputs_start: 1,
        prot_i2c: 1,
        prot_1wire: 1,
        additional_options: 1,
        load_status: 1,
        custom_device_name: 1,
        po_tlog27_support: 1,
        sensor_list: 1,
        web_interface: 1,
        fail_safe_settings: 1,
        joystick_hat_switch: 1,
        pulse_engine: 1,
        pulse_engine_v2: 1,
        easy_sensors: 1,
    };

    // Create a minimal DeviceData for testing
    let device_data = DeviceData {
        device_type_id: 10,
        serial_number: 12345,
        device_name: [0; 30],
        device_type_name: [0; 30],
        build_date: [0; 12],
        activation_code: [0; 8],
        firmware_version_major: 1,
        firmware_version_minor: 2,
        firmware_revision: 0,
        user_id: 0,
        device_type: 7,
        activated_options: 0,
        device_lock_status: 0,
        hw_type: 0,
        fw_type: 0,
        product_id: 0,
        secondary_firmware_version_major: 0,
        secondary_firmware_version_minor: 0,
        device_is_bootloader: 0,
    };

    let state = DeviceState::new(device_info.clone(), device_data.clone());

    // Test basic properties
    assert_eq!(state.device_data.serial_number, 12345);
    assert_eq!(state.device_data.firmware_version_major, 1);
    assert_eq!(state.device_data.firmware_version_minor, 2);
    assert_eq!(state.device_data.device_type, 7);
    assert_eq!(state.status, ThreadStatus::Stopped);
    assert!(state.error_message.is_none());
}

#[test]
fn test_shared_device_state() {
    // Create a minimal DeviceInfo for testing
    let device_info = DeviceInfo {
        pin_count: 55,
        pwm_count: 6,
        basic_encoder_count: 25,
        encoders_count: 25,
        fast_encoders: 0,
        ultra_fast_encoders: 0,
        pwm_internal_frequency: 10000,
        analog_inputs: 7,
        key_mapping: 1,
        triggered_key_mapping: 1,
        key_repeat_delay: 0,
        digital_counters: 1,
        joystick_button_axis_mapping: 1,
        joystick_analog_to_digital_mapping: 1,
        macros: 1,
        matrix_keyboard: 1,
        matrix_keyboard_triggered_mapping: 1,
        lcd: 1,
        matrix_led: 1,
        connection_signal: 1,
        po_ext_bus: 1,
        po_net: 1,
        analog_filtering: 1,
        init_outputs_start: 1,
        prot_i2c: 1,
        prot_1wire: 1,
        additional_options: 1,
        load_status: 1,
        custom_device_name: 1,
        po_tlog27_support: 1,
        sensor_list: 1,
        web_interface: 1,
        fail_safe_settings: 1,
        joystick_hat_switch: 1,
        pulse_engine: 1,
        pulse_engine_v2: 1,
        easy_sensors: 1,
    };

    // Create a minimal DeviceData for testing
    let device_data = DeviceData {
        device_type_id: 10,
        serial_number: 12345,
        device_name: [0; 30],
        device_type_name: [0; 30],
        build_date: [0; 12],
        activation_code: [0; 8],
        firmware_version_major: 1,
        firmware_version_minor: 2,
        firmware_revision: 0,
        user_id: 0,
        device_type: 7,
        activated_options: 0,
        device_lock_status: 0,
        hw_type: 0,
        fw_type: 0,
        product_id: 0,
        secondary_firmware_version_major: 0,
        secondary_firmware_version_minor: 0,
        device_is_bootloader: 0,
    };

    let shared_state = Arc::new(SharedDeviceState::new(
        device_info.clone(),
        device_data.clone(),
    ));

    // Test status
    assert_eq!(shared_state.status(), ThreadStatus::Stopped);

    // Test set_running
    shared_state.set_running(true);
    assert_eq!(shared_state.status(), ThreadStatus::Running);

    // Test set_paused
    shared_state.set_paused(true);
    assert_eq!(shared_state.status(), ThreadStatus::Paused);

    // Test set_paused(false)
    shared_state.set_paused(false);
    assert_eq!(shared_state.status(), ThreadStatus::Running);

    // Test set_running(false)
    shared_state.set_running(false);
    assert_eq!(shared_state.status(), ThreadStatus::Stopped);

    // Test custom values
    shared_state.set_custom_value("test_key", "test_value");
    assert_eq!(
        shared_state.get_custom_value("test_key").unwrap(),
        "test_value"
    );
    assert!(shared_state.get_custom_value("non_existent_key").is_none());

    // Test error message
    shared_state.set_error(Some("Test error".to_string()));
    assert_eq!(shared_state.get_error().unwrap(), "Test error");
    shared_state.set_error(None);
    assert!(shared_state.get_error().is_none());
}

#[test]
fn test_state_observer() {
    // Create a minimal DeviceInfo for testing
    let device_info = DeviceInfo {
        pin_count: 55,
        pwm_count: 6,
        basic_encoder_count: 25,
        encoders_count: 25,
        fast_encoders: 0,
        ultra_fast_encoders: 0,
        pwm_internal_frequency: 10000,
        analog_inputs: 7,
        key_mapping: 1,
        triggered_key_mapping: 1,
        key_repeat_delay: 0,
        digital_counters: 1,
        joystick_button_axis_mapping: 1,
        joystick_analog_to_digital_mapping: 1,
        macros: 1,
        matrix_keyboard: 1,
        matrix_keyboard_triggered_mapping: 1,
        lcd: 1,
        matrix_led: 1,
        connection_signal: 1,
        po_ext_bus: 1,
        po_net: 1,
        analog_filtering: 1,
        init_outputs_start: 1,
        prot_i2c: 1,
        prot_1wire: 1,
        additional_options: 1,
        load_status: 1,
        custom_device_name: 1,
        po_tlog27_support: 1,
        sensor_list: 1,
        web_interface: 1,
        fail_safe_settings: 1,
        joystick_hat_switch: 1,
        pulse_engine: 1,
        pulse_engine_v2: 1,
        easy_sensors: 1,
    };

    // Create a minimal DeviceData for testing
    let device_data = DeviceData {
        device_type_id: 10,
        serial_number: 12345,
        device_name: [0; 30],
        device_type_name: [0; 30],
        build_date: [0; 12],
        activation_code: [0; 8],
        firmware_version_major: 1,
        firmware_version_minor: 2,
        firmware_revision: 0,
        user_id: 0,
        device_type: 7,
        activated_options: 0,
        device_lock_status: 0,
        hw_type: 0,
        fw_type: 0,
        product_id: 0,
        secondary_firmware_version_major: 0,
        secondary_firmware_version_minor: 0,
        device_is_bootloader: 0,
    };

    let shared_state = Arc::new(SharedDeviceState::new(
        device_info.clone(),
        device_data.clone(),
    ));
    let observer = crate::observer::StateObserver::new(1, shared_state.clone());

    // Test wait_for_change with timeout
    let change = observer.wait_for_change(Duration::from_millis(10));
    assert!(change.is_none());

    // Test check_for_change
    let change = observer.check_for_change();
    assert!(change.is_none());

    // Trigger a state change
    shared_state.set_running(true);

    // Test process_all_changes
    let mut changes = Vec::new();
    observer.process_all_changes(|change| {
        changes.push(change);
    });

    assert_eq!(changes.len(), 1);
    match &changes[0] {
        StateChangeType::ThreadStatus { status } => {
            assert_eq!(*status, ThreadStatus::Running);
        }
        _ => panic!("Unexpected state change type"),
    }
}

#[test]
fn test_device_command() {
    // Test DeviceCommand variants
    let cmd1 = DeviceCommand::Start;
    let cmd2 = DeviceCommand::Pause;
    let cmd3 = DeviceCommand::Terminate;
    let cmd4 = DeviceCommand::Restart;
    let cmd5 = DeviceCommand::GetStatus;
    let cmd6 = DeviceCommand::SetDigitalOutput {
        pin: 1,
        value: true,
    };
    let cmd7 = DeviceCommand::SetAnalogOutput {
        pin: 2,
        value: 1000,
    };
    let cmd8 = DeviceCommand::SetPwmDuty {
        channel: 0,
        duty: 2048,
    };
    let cmd9 = DeviceCommand::ConfigureEncoder {
        encoder_index: 0,
        pin_a: 1,
        pin_b: 2,
        enabled: true,
        sampling_4x: false,
    };
    let cmd10 = DeviceCommand::ResetDigitalCounter { pin: 3 };
    let cmd11 = DeviceCommand::Custom {
        request_type: 0x01,
        param1: 0x02,
        param2: 0x03,
        param3: 0x04,
        param4: 0x05,
    };
    let cmd12 = DeviceCommand::SetLogLevel(LevelFilter::Debug);

    // Just verify that we can create all command variants
    assert!(matches!(cmd1, DeviceCommand::Start));
    assert!(matches!(cmd2, DeviceCommand::Pause));
    assert!(matches!(cmd3, DeviceCommand::Terminate));
    assert!(matches!(cmd4, DeviceCommand::Restart));
    assert!(matches!(cmd5, DeviceCommand::GetStatus));
    assert!(matches!(
        cmd6,
        DeviceCommand::SetDigitalOutput {
            pin: 1,
            value: true
        }
    ));
    assert!(matches!(
        cmd7,
        DeviceCommand::SetAnalogOutput {
            pin: 2,
            value: 1000
        }
    ));
    assert!(matches!(
        cmd8,
        DeviceCommand::SetPwmDuty {
            channel: 0,
            duty: 2048
        }
    ));
    assert!(matches!(
        cmd9,
        DeviceCommand::ConfigureEncoder {
            encoder_index: 0,
            pin_a: 1,
            pin_b: 2,
            enabled: true,
            sampling_4x: false,
        }
    ));
    assert!(matches!(
        cmd10,
        DeviceCommand::ResetDigitalCounter { pin: 3 }
    ));
    assert!(matches!(
        cmd11,
        DeviceCommand::Custom {
            request_type: 0x01,
            param1: 0x02,
            param2: 0x03,
            param3: 0x04,
            param4: 0x05,
        }
    ));
    assert!(matches!(
        cmd12,
        DeviceCommand::SetLogLevel(LevelFilter::Debug)
    ));
}

#[test]
fn test_thread_error() {
    // Test ThreadError variants
    let err1 = ThreadError::ThreadNotFound(1);
    let err2 = ThreadError::ThreadAlreadyExists(2);
    let err3 = ThreadError::ThreadCreationFailed("Failed to create thread".to_string());
    let err4 = ThreadError::CommandSendFailed("Failed to send command".to_string());
    let err5 = ThreadError::DeviceError(pokeys_lib::PoKeysError::DeviceNotFound);
    let err6 = ThreadError::IoError(std::io::Error::other("IO error"));
    let err7 = ThreadError::ThreadJoinError;
    let err8 = ThreadError::InvalidCommand("Invalid command".to_string());
    let err9 = ThreadError::Timeout;
    let err10 = ThreadError::NotSupported;
    let err11 = ThreadError::InvalidParameter("Invalid parameter".to_string());
    let err12 = ThreadError::OperationFailed("Operation failed".to_string());
    let err13 = ThreadError::LockPoisoned("Lock poisoned".to_string());
    let err14 = ThreadError::ChannelReceiveError("Channel receive error".to_string());
    let err15 = ThreadError::ChannelSendError("Channel send error".to_string());
    let err16 = ThreadError::StateError("State error".to_string());
    let err17 = ThreadError::ConnectionError("Connection error".to_string());
    let err18 = ThreadError::InitializationError("Initialization error".to_string());
    let err19 = ThreadError::ConfigurationError("Configuration error".to_string());

    // Just verify that we can create all error variants
    assert!(matches!(err1, ThreadError::ThreadNotFound(1)));
    assert!(matches!(err2, ThreadError::ThreadAlreadyExists(2)));
    assert!(matches!(err3, ThreadError::ThreadCreationFailed(_)));
    assert!(matches!(err4, ThreadError::CommandSendFailed(_)));
    assert!(matches!(err5, ThreadError::DeviceError(_)));
    assert!(matches!(err6, ThreadError::IoError(_)));
    assert!(matches!(err7, ThreadError::ThreadJoinError));
    assert!(matches!(err8, ThreadError::InvalidCommand(_)));
    assert!(matches!(err9, ThreadError::Timeout));
    assert!(matches!(err10, ThreadError::NotSupported));
    assert!(matches!(err11, ThreadError::InvalidParameter(_)));
    assert!(matches!(err12, ThreadError::OperationFailed(_)));
    assert!(matches!(err13, ThreadError::LockPoisoned(_)));
    assert!(matches!(err14, ThreadError::ChannelReceiveError(_)));
    assert!(matches!(err15, ThreadError::ChannelSendError(_)));
    assert!(matches!(err16, ThreadError::StateError(_)));
    assert!(matches!(err17, ThreadError::ConnectionError(_)));
    assert!(matches!(err18, ThreadError::InitializationError(_)));
    assert!(matches!(err19, ThreadError::ConfigurationError(_)));

    // Test error display
    assert_eq!(err1.to_string(), "Thread not found: 1");
    assert_eq!(err2.to_string(), "Thread already exists: 2");
    assert_eq!(
        err3.to_string(),
        "Thread creation failed: Failed to create thread"
    );
}

#[test]
fn test_logger() {
    // Test SimpleLogger
    let mut logger = SimpleLogger::new(LevelFilter::Info);
    assert_eq!(logger.level(), LevelFilter::Info);
    logger.set_level(LevelFilter::Debug);
    assert_eq!(logger.level(), LevelFilter::Debug);

    // Test MockLogger
    let mut mock_logger = MockLogger::new(LevelFilter::Info);
    assert_eq!(mock_logger.level(), LevelFilter::Info);
    mock_logger.set_level(LevelFilter::Debug);
    assert_eq!(mock_logger.level(), LevelFilter::Debug);
}
