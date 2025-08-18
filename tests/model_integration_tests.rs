use pokeys_lib::models::{DeviceModel, PinModel};
use pokeys_thread::{ThreadController, ThreadControllerBuilder};
use std::collections::HashMap;
use std::fs;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_model_monitoring() {
    // This test will only run if a device is connected
    // Otherwise, it will be skipped
    if pokeys_lib::enumerate_usb_devices().unwrap_or(0) == 0 {
        println!("No device connected, skipping test");
        return;
    }

    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a model
    let mut model = DeviceModel {
        name: "TestDevice".to_string(),
        pins: HashMap::new(),
    };

    // Add some pins
    model.pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "DigitalOutput".to_string()],
            active: true,
        },
    );

    // Write the model file
    let file_path = dir.path().join("PoKeys56U.yaml");
    let yaml = serde_yaml::to_string(&model).unwrap();
    fs::write(&file_path, yaml).unwrap();

    // Create a thread controller
    let mut controller = ThreadControllerBuilder::new().build();

    // Discover USB devices
    let devices = controller.discover_usb_devices().unwrap();

    if !devices.is_empty() {
        // Start a thread for the first device
        let thread_id = controller.start_usb_device_thread(devices[0]).unwrap();

        // Start model monitoring
        controller
            .start_model_monitoring(thread_id, Some(dir.path().to_path_buf()))
            .unwrap();

        // Wait for the model to be loaded
        thread::sleep(Duration::from_millis(500));

        // Get the device state
        let state = controller.get_state(thread_id).unwrap();

        // Check if the model was loaded
        assert!(state.model.is_some(), "Model was not loaded");

        // Check the model
        let loaded_model = state.model.unwrap();
        assert_eq!(loaded_model.name, "TestDevice");
        assert_eq!(loaded_model.pins.len(), 1);
        assert!(loaded_model.pins.contains_key(&1));

        // Update the model
        let mut updated_model = model.clone();
        updated_model.pins.insert(
            2,
            PinModel {
                capabilities: vec!["DigitalInput".to_string(), "AnalogInput".to_string()],
                active: true,
            },
        );

        // Write the updated model file
        let yaml = serde_yaml::to_string(&updated_model).unwrap();
        fs::write(&file_path, yaml).unwrap();

        // Wait for the model to be updated
        thread::sleep(Duration::from_millis(500));

        // Get the device state again
        let state = controller.get_state(thread_id).unwrap();

        // Check if the model was updated
        assert!(state.model.is_some(), "Model was not updated");

        // Check the updated model
        let loaded_model = state.model.unwrap();
        assert_eq!(loaded_model.name, "TestDevice");
        assert_eq!(loaded_model.pins.len(), 2);
        assert!(loaded_model.pins.contains_key(&1));
        assert!(loaded_model.pins.contains_key(&2));

        // Stop model monitoring
        controller.stop_model_monitoring(thread_id).unwrap();

        // Stop the thread
        controller.stop_all().unwrap();
    }
}

#[test]
fn test_update_device_model() {
    // This test will only run if a device is connected
    // Otherwise, it will be skipped
    if pokeys_lib::enumerate_usb_devices().unwrap_or(0) == 0 {
        println!("No device connected, skipping test");
        return;
    }

    // Create a model
    let mut model = DeviceModel {
        name: "TestDevice".to_string(),
        pins: HashMap::new(),
    };

    // Add some pins
    model.pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "DigitalOutput".to_string()],
            active: true,
        },
    );

    // Create a thread controller
    let mut controller = ThreadControllerBuilder::new().build();

    // Discover USB devices
    let devices = controller.discover_usb_devices().unwrap();

    if !devices.is_empty() {
        // Start a thread for the first device
        let thread_id = controller.start_usb_device_thread(devices[0]).unwrap();

        // Update the device model
        controller
            .update_device_model(thread_id, model.clone())
            .unwrap();

        // Wait for the model to be updated
        thread::sleep(Duration::from_millis(500));

        // Get the device state
        let state = controller.get_state(thread_id).unwrap();

        // Check if the model was updated
        assert!(state.model.is_some(), "Model was not updated");

        // Check the model
        let loaded_model = state.model.unwrap();
        assert_eq!(loaded_model.name, "TestDevice");
        assert_eq!(loaded_model.pins.len(), 1);
        assert!(loaded_model.pins.contains_key(&1));

        // Stop the thread
        controller.stop_all().unwrap();
    }
}
