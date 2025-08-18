//! Tests for the state observer

#[cfg(test)]
mod tests {
    use pokeys_thread::{SharedDeviceState, StateChangeType, StateObserver, ThreadStatus};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_state_observer() {
        // Create a minimal DeviceInfo for testing
        let device_info = pokeys_lib::DeviceInfo {
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
        let device_data = pokeys_lib::DeviceData {
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

        let shared_state = Arc::new(SharedDeviceState::new(device_info, device_data));

        // Create a state observer
        let observer = StateObserver::new(1, shared_state.clone());

        // Start a thread to make state changes
        let state_thread = thread::spawn(move || {
            // Wait a bit before making changes
            thread::sleep(Duration::from_millis(100));

            // Set a custom value
            shared_state.set_custom_value("test", "value");

            // Wait a bit
            thread::sleep(Duration::from_millis(100));

            // Set the thread as running
            shared_state.set_running(true);

            // Wait a bit
            thread::sleep(Duration::from_millis(100));

            // Set the thread as paused
            shared_state.set_paused(true);

            // Wait a bit
            thread::sleep(Duration::from_millis(100));

            // Set an error
            shared_state.set_error(Some("Test error".to_string()));
        });

        // Wait for and process state changes
        let mut changes = Vec::new();
        let start_time = std::time::Instant::now();
        while start_time.elapsed() < Duration::from_secs(1) {
            if let Some(change_type) = observer.wait_for_change(Duration::from_millis(100)) {
                changes.push(change_type);
            }
        }

        // Wait for the state thread to finish
        state_thread.join().unwrap();

        // Check that we received the expected changes
        assert!(changes.iter().any(|change| matches!(change,
            StateChangeType::CustomValue { key, value } if key == "test" && value == "value")));

        assert!(changes.iter().any(|change| matches!(change,
            StateChangeType::ThreadStatus { status } if *status == ThreadStatus::Running)));

        assert!(changes.iter().any(|change| matches!(change,
            StateChangeType::ThreadStatus { status } if *status == ThreadStatus::Paused)));

        assert!(changes.iter().any(|change| matches!(change,
            StateChangeType::Error { message } if message.as_ref().unwrap() == "Test error")));

        // Check the final state
        assert_eq!(observer.shared_state().status(), ThreadStatus::Paused);
        assert_eq!(
            observer.shared_state().get_custom_value("test"),
            Some("value".to_string())
        );
        assert_eq!(
            observer.shared_state().get_error(),
            Some("Test error".to_string())
        );
    }
}
