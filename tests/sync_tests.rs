//! Tests for the device sync

#[cfg(test)]
mod tests {
    use pokeys_thread::{DeviceSync, SharedDeviceState};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    #[ignore] // Ignore by default as it requires actual hardware
    fn test_device_sync() {
        // Initialize logging
        let _ = env_logger::builder().is_test(true).try_init();

        // Create a mock device state
        let device_info = pokeys_lib::DeviceInfo::default();
        let device_data = pokeys_lib::DeviceData::default();
        let shared_state = Arc::new(SharedDeviceState::new(device_info, device_data));

        // Create a device sync
        let mut device_sync = DeviceSync::new(1, shared_state.clone(), 100);

        // Check if it's time to sync
        assert!(device_sync.should_sync());

        // Wait a bit
        thread::sleep(Duration::from_millis(10));

        // Check if it's still time to sync
        assert!(device_sync.should_sync());

        // Set a longer sync interval
        device_sync.set_sync_interval(1000);

        // Check if it's time to sync
        assert!(!device_sync.should_sync());

        // Wait a bit
        thread::sleep(Duration::from_millis(1100));

        // Check if it's time to sync
        assert!(device_sync.should_sync());
    }
}
