//! Tests for the thread controller

#[cfg(test)]
mod tests {
    use pokeys_thread::{DeviceCommand, ThreadController, ThreadControllerBuilder, ThreadStatus};
    use std::thread;
    use std::time::Duration;

    #[test]
    #[ignore] // Ignore by default as it requires actual hardware
    fn test_usb_device_discovery() {
        // Initialize logging
        let _ = env_logger::builder().is_test(true).try_init();

        // Create a thread controller
        let mut controller = ThreadControllerBuilder::new()
            .default_refresh_interval(100)
            .build();

        // Discover USB devices
        let devices = controller.discover_usb_devices();
        assert!(
            devices.is_ok(),
            "Failed to discover USB devices: {:?}",
            devices.err()
        );

        let devices = devices.unwrap();
        println!("Found {} USB devices", devices.len());

        // If no devices are found, skip the rest of the test
        if devices.is_empty() {
            return;
        }

        // Start a thread for the first device
        let thread_id = controller.start_usb_device_thread(devices[0]);
        assert!(
            thread_id.is_ok(),
            "Failed to start device thread: {:?}",
            thread_id.err()
        );

        let thread_id = thread_id.unwrap();

        // Wait a bit for the thread to start
        thread::sleep(Duration::from_millis(100));

        // Check the status
        let status = controller.get_status(thread_id);
        assert!(
            status.is_ok(),
            "Failed to get thread status: {:?}",
            status.err()
        );
        assert_eq!(status.unwrap(), ThreadStatus::Running);

        // Send a command to get the status
        let result = controller.send_command(thread_id, DeviceCommand::GetStatus);
        assert!(result.is_ok(), "Failed to send command: {:?}", result.err());

        // Wait a bit for the command to be processed
        thread::sleep(Duration::from_millis(100));

        // Get the state
        let state = controller.get_state(thread_id);
        assert!(
            state.is_ok(),
            "Failed to get thread state: {:?}",
            state.err()
        );

        // Stop the thread
        let result = controller.send_command(thread_id, DeviceCommand::Terminate);
        assert!(
            result.is_ok(),
            "Failed to send terminate command: {:?}",
            result.err()
        );

        // Wait a bit for the command to be processed
        thread::sleep(Duration::from_millis(100));

        // Stop all threads
        let result = controller.stop_all();
        assert!(
            result.is_ok(),
            "Failed to stop all threads: {:?}",
            result.err()
        );
    }

    #[test]
    #[ignore] // Ignore by default as it requires actual hardware
    fn test_network_device_discovery() {
        // Initialize logging
        let _ = env_logger::builder().is_test(true).try_init();

        // Create a thread controller
        let mut controller = ThreadControllerBuilder::new()
            .default_refresh_interval(100)
            .build();

        // Discover network devices
        let devices = controller.discover_network_devices(2000);
        assert!(
            devices.is_ok(),
            "Failed to discover network devices: {:?}",
            devices.err()
        );

        let devices = devices.unwrap();
        println!("Found {} network devices", devices.len());

        // If no devices are found, skip the rest of the test
        if devices.is_empty() {
            return;
        }

        // Start a thread for the first device
        let thread_id = controller.start_network_device_thread(devices[0].clone());
        assert!(
            thread_id.is_ok(),
            "Failed to start device thread: {:?}",
            thread_id.err()
        );

        let thread_id = thread_id.unwrap();

        // Wait a bit for the thread to start
        thread::sleep(Duration::from_millis(100));

        // Check the status
        let status = controller.get_status(thread_id);
        assert!(
            status.is_ok(),
            "Failed to get thread status: {:?}",
            status.err()
        );
        assert_eq!(status.unwrap(), ThreadStatus::Running);

        // Stop the thread
        let result = controller.send_command(thread_id, DeviceCommand::Terminate);
        assert!(
            result.is_ok(),
            "Failed to send terminate command: {:?}",
            result.err()
        );

        // Wait a bit for the command to be processed
        thread::sleep(Duration::from_millis(100));

        // Stop all threads
        let result = controller.stop_all();
        assert!(
            result.is_ok(),
            "Failed to stop all threads: {:?}",
            result.err()
        );
    }
}
