//! Tests for the device worker

#[cfg(test)]
mod tests {
    use pokeys_thread::{DeviceCommand, ThreadStatus, ThreadWorkerBuilder};
    use std::thread;
    use std::time::Duration;

    #[test]
    #[ignore] // Ignore by default as it requires actual hardware
    fn test_usb_device_worker() {
        // Initialize logging
        let _ = env_logger::builder().is_test(true).try_init();

        // Create a device worker for the first USB device
        let builder = ThreadWorkerBuilder::new(1).refresh_interval(100);
        let worker = builder.build_usb_device(0);

        // Check if the worker was created successfully
        assert!(
            worker.is_ok(),
            "Failed to create device worker: {:?}",
            worker.err()
        );
        let worker = worker.unwrap();

        // Check the initial status
        assert_eq!(worker.status(), ThreadStatus::Running);

        // Send a command to get the status
        let result = worker.send_command(DeviceCommand::GetStatus);
        assert!(result.is_ok(), "Failed to send command: {:?}", result.err());

        // Wait a bit for the command to be processed
        thread::sleep(Duration::from_millis(100));

        // Pause the worker
        let result = worker.send_command(DeviceCommand::Pause);
        assert!(
            result.is_ok(),
            "Failed to send pause command: {:?}",
            result.err()
        );

        // Wait a bit for the command to be processed
        thread::sleep(Duration::from_millis(100));

        // Check the status
        assert_eq!(worker.status(), ThreadStatus::Paused);

        // Resume the worker
        let result = worker.send_command(DeviceCommand::Start);
        assert!(
            result.is_ok(),
            "Failed to send start command: {:?}",
            result.err()
        );

        // Wait a bit for the command to be processed
        thread::sleep(Duration::from_millis(100));

        // Check the status
        assert_eq!(worker.status(), ThreadStatus::Running);

        // Stop the worker
        let result = worker.send_command(DeviceCommand::Terminate);
        assert!(
            result.is_ok(),
            "Failed to send terminate command: {:?}",
            result.err()
        );

        // Wait a bit for the command to be processed
        thread::sleep(Duration::from_millis(100));

        // Check the status
        assert_eq!(worker.status(), ThreadStatus::Stopped);
    }
}
