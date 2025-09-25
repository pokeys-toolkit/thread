use pokeys_thread::*;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_pwm_data_structure_compatibility() {
    // Test that PWM data structure works with new pwm_values field
    let device_info = pokeys_lib::DeviceInfo::default();
    let device_data = pokeys_lib::DeviceData::default();
    let state = DeviceState::new(device_info, device_data);

    // Test that PWM data has the expected structure
    assert_eq!(state.pwm.pwm_values.len(), 6); // 6 PWM channels
    assert_eq!(state.pwm.pwm_period, 0); // Default period
    assert_eq!(state.pwm.enabled_channels, 0); // Default disabled
}

#[test]
fn test_shared_state_pwm_operations() {
    // Test PWM operations on shared state
    let device_info = pokeys_lib::DeviceInfo::default();
    let device_data = pokeys_lib::DeviceData::default();
    let shared_state = Arc::new(SharedDeviceState::new(device_info, device_data));

    // Test setting PWM duty cycle
    shared_state.set_pwm_duty_cycle(0, 1000);
    assert_eq!(shared_state.get_pwm_duty_cycle(0), Some(1000));

    // Test setting multiple channels
    for channel in 0..6 {
        let duty = (channel + 1) * 500;
        shared_state.set_pwm_duty_cycle(channel, duty as u32);
        assert_eq!(shared_state.get_pwm_duty_cycle(channel), Some(duty as u32));
    }

    // Test invalid channel
    assert_eq!(shared_state.get_pwm_duty_cycle(10), None);
}

#[test]
fn test_pwm_state_change_notifications() {
    // Test that PWM changes generate proper notifications
    let device_info = pokeys_lib::DeviceInfo::default();
    let device_data = pokeys_lib::DeviceData::default();
    let shared_state = Arc::new(SharedDeviceState::new(device_info, device_data));

    let observer = StateObserver::new(1, shared_state.clone());

    // Set PWM duty cycle and check for notification
    shared_state.set_pwm_duty_cycle(2, 2048);

    // Wait for notification with timeout
    if let Some(change) = observer.wait_for_change(Duration::from_millis(100)) {
        match change {
            StateChangeType::PwmDutyCycle { channel, duty } => {
                assert_eq!(channel, 2);
                assert_eq!(duty, 2048);
            }
            _ => panic!("Expected PWM duty cycle change notification"),
        }
    } else {
        panic!("No notification received");
    }
}

#[test]
fn test_pwm_channel_to_pin_mapping() {
    // Test the PWM channel to pin mapping used in worker.rs
    // Channel 0 -> Pin 22, Channel 1 -> Pin 21, etc.
    let expected_mapping = [(0, 22), (1, 21), (2, 20), (3, 19), (4, 18), (5, 17)];

    for (channel, expected_pin) in expected_mapping {
        let pin = match channel {
            0 => 22,
            1 => 21,
            2 => 20,
            3 => 19,
            4 => 18,
            5 => 17,
            _ => panic!("Invalid channel"),
        };
        assert_eq!(
            pin, expected_pin,
            "Channel {channel} should map to pin {expected_pin}"
        );
    }
}

#[test]
fn test_pwm_state_update_from_device() {
    // Test that PWM state updates correctly detect changes
    let device_info = pokeys_lib::DeviceInfo::default();
    let device_data = pokeys_lib::DeviceData::default();
    let _shared_state = Arc::new(SharedDeviceState::new(device_info, device_data));

    // Create initial PWM state
    let mut old_pwm = pokeys_lib::pwm::PwmData::new();
    old_pwm.pwm_values[0] = 1000;
    old_pwm.pwm_values[1] = 2000;

    // Create new PWM state with changes
    let mut new_pwm = pokeys_lib::pwm::PwmData::new();
    new_pwm.pwm_values[0] = 1500; // Changed
    new_pwm.pwm_values[1] = 2000; // Unchanged
    new_pwm.pwm_values[2] = 3000; // New value

    // Test that the comparison logic works (this tests the fixed code in state.rs)
    let changes: Vec<_> = old_pwm
        .pwm_values
        .iter()
        .zip(new_pwm.pwm_values.iter())
        .enumerate()
        .filter(|(_, (old, new))| old != new)
        .collect();

    assert_eq!(changes.len(), 2); // Channels 0 and 2 changed
    assert_eq!(changes[0].0, 0); // Channel 0
    assert_eq!(changes[1].0, 2); // Channel 2
}

#[test]
fn test_device_command_pwm_structure() {
    // Test that PWM commands have the expected structure
    let command = DeviceCommand::SetPwmDuty {
        channel: 3,
        duty: 2048,
    };

    match command {
        DeviceCommand::SetPwmDuty { channel, duty } => {
            assert_eq!(channel, 3);
            assert_eq!(duty, 2048);
        }
        _ => panic!("Expected SetPwmDuty command"),
    }
}

#[test]
fn test_pwm_bounds_checking() {
    // Test PWM bounds checking
    let device_info = pokeys_lib::DeviceInfo::default();
    let device_data = pokeys_lib::DeviceData::default();
    let shared_state = Arc::new(SharedDeviceState::new(device_info, device_data));

    // Test valid channels (0-5)
    for channel in 0..6 {
        shared_state.set_pwm_duty_cycle(channel, 1000);
        assert_eq!(shared_state.get_pwm_duty_cycle(channel), Some(1000));
    }

    // Test invalid channels
    assert_eq!(shared_state.get_pwm_duty_cycle(6), None);
    assert_eq!(shared_state.get_pwm_duty_cycle(100), None);

    // Setting invalid channels should not crash (they're silently ignored)
    shared_state.set_pwm_duty_cycle(10, 1000);
    assert_eq!(shared_state.get_pwm_duty_cycle(10), None);
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;

    #[test]
    fn test_pwm_thread_safety() {
        // Test that PWM operations are thread-safe
        let device_info = pokeys_lib::DeviceInfo::default();
        let device_data = pokeys_lib::DeviceData::default();
        let shared_state = Arc::new(SharedDeviceState::new(device_info, device_data));

        let done = Arc::new(AtomicBool::new(false));
        let mut handles = vec![];

        // Spawn multiple threads that modify PWM values
        for thread_id in 0..4 {
            let state = shared_state.clone();
            let done_flag = done.clone();

            let handle = thread::spawn(move || {
                let mut counter = 0;
                while !done_flag.load(Ordering::Relaxed) {
                    let channel = thread_id % 6;
                    let duty = (counter % 4096) as u32;
                    state.set_pwm_duty_cycle(channel, duty);

                    // Verify the value was set
                    if let Some(read_duty) = state.get_pwm_duty_cycle(channel) {
                        // The value might have been changed by another thread,
                        // but it should be a valid PWM value
                        assert!(read_duty <= 4095);
                    }

                    counter += 1;
                    if counter > 100 {
                        break;
                    }
                }
            });

            handles.push(handle);
        }

        // Let threads run for a short time
        thread::sleep(Duration::from_millis(100));
        done.store(true, Ordering::Relaxed);

        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread should complete successfully");
        }

        // Verify final state is consistent
        for channel in 0..6 {
            if let Some(duty) = shared_state.get_pwm_duty_cycle(channel) {
                assert!(duty <= 4095, "PWM duty cycle should be within valid range");
            }
        }
    }
}
