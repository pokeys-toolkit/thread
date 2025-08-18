#![allow(clippy::uninlined_format_args)]
use log::{debug, error, info, warn, LevelFilter};
use pokeys_thread::{
    DeviceCommand, DeviceOperations, SimpleLogger, ThreadController, ThreadControllerBuilder,
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    // Initialize the logger
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();

    info!("Starting logging example");

    // Create a simple logger
    let logger = Arc::new(SimpleLogger::new(LevelFilter::Debug));

    // Create a thread controller with the logger
    let mut controller = ThreadControllerBuilder::new()
        .default_refresh_interval(100)
        .with_logger(logger.clone())
        .build();

    let mut thread_id = None;

    // Try USB devices first
    match controller.discover_usb_devices() {
        Ok(devices) => {
            info!("Found {} USB devices", devices.len());

            if !devices.is_empty() {
                // Start a thread for the first USB device
                let device_index = devices[0];
                match controller.start_usb_device_thread(device_index) {
                    Ok(id) => {
                        info!("Started thread {} for USB device {}", id, device_index);
                        thread_id = Some(id);
                    }
                    Err(e) => {
                        error!(
                            "Failed to start thread for USB device {}: {}",
                            device_index, e
                        );
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to discover USB devices: {}", e);
        }
    }

    // If no USB device was started, try network devices
    if thread_id.is_none() {
        match controller.discover_network_devices(2000) {
            Ok(devices) => {
                info!("Found {} network devices", devices.len());

                if !devices.is_empty() {
                    // Start a thread for the first network device
                    match controller.start_network_device_thread(devices[0].clone()) {
                        Ok(id) => {
                            info!(
                                "Started thread {} for network device with serial {}",
                                id, devices[0].serial_number
                            );
                            thread_id = Some(id);
                        }
                        Err(e) => {
                            error!(
                                "Failed to start thread for network device {}: {}",
                                devices[0].serial_number, e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to discover network devices: {}", e);
            }
        }
    }

    if let Some(thread_id) = thread_id {
        // Wait for the thread to initialize
        thread::sleep(Duration::from_millis(500));

        // Get the device state
        match controller.get_state(thread_id) {
            Ok(state) => {
                info!("Device state:");
                info!("  Serial number: {}", state.device_data.serial_number);
                info!(
                    "  Firmware version: {}.{}",
                    state.device_data.firmware_version_major,
                    state.device_data.firmware_version_minor
                );
                debug!("Device pins: {} configured", state.pins.len());
            }
            Err(e) => {
                error!("Failed to get device state: {}", e);
            }
        }

        // Set log level to Info
        if let Err(e) = controller.set_thread_log_level(thread_id, LevelFilter::Info) {
            error!("Failed to set thread log level: {}", e);
        }

        // Toggle some digital outputs with different log levels
        for i in 0..5 {
            let pin = 1 + i;

            // Set output high
            if let Err(e) = controller.set_digital_output(thread_id, pin, true) {
                error!("Failed to set digital output {} high: {}", pin, e);
            }

            thread::sleep(Duration::from_millis(200));

            // Set output low
            if let Err(e) = controller.set_digital_output(thread_id, pin, false) {
                error!("Failed to set digital output {} low: {}", pin, e);
            }

            thread::sleep(Duration::from_millis(200));
        }

        // Set global log level to Trace
        if let Err(e) = controller.set_global_log_level(LevelFilter::Trace) {
            error!("Failed to set global log level: {}", e);
        }

        // Read some inputs
        for i in 0..5 {
            let pin = 1 + i;

            match controller.get_digital_input(thread_id, pin) {
                Ok(value) => {
                    info!("Digital input {} value: {}", pin, value);
                }
                Err(e) => {
                    error!("Failed to get digital input {}: {}", pin, e);
                }
            }

            thread::sleep(Duration::from_millis(100));
        }

        // Stop the thread
        if let Err(e) = controller.send_command(thread_id, DeviceCommand::Terminate) {
            error!("Failed to terminate thread {}: {}", thread_id, e);
        }

        // Wait for the thread to terminate
        thread::sleep(Duration::from_millis(500));
    } else {
        warn!("No PoKeys devices found (USB or network)");
    }

    info!("Logging example completed");
}
