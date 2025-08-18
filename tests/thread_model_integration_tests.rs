use pokeys_lib::models::{DeviceModel, PinModel};
use pokeys_thread::{DeviceOperations, ThreadController, ThreadControllerBuilder};
use std::collections::HashMap;
use std::fs;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

// This test requires a connected device, so it will be skipped if no device is found
#[test]
fn test_automatic_model_monitoring() {
    // Skip if no device is connected
    if pokeys_lib::enumerate_usb_devices().unwrap_or(0) == 0 {
        println!("No device connected, skipping test");
        return;
    }

    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a model
    let mut model = DeviceModel {
        name: "PoKeys56U".to_string(),
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

    model.pins.insert(
        2,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "AnalogInput".to_string()],
            active: true,
        },
    );

    // Write the model file
    let file_path = dir.path().join("PoKeys56U.yaml");
    let yaml = serde_yaml::to_string(&model).unwrap();
    fs::write(&file_path, yaml).unwrap();

    // Create a thread controller with the custom model directory
    let mut controller = ThreadControllerBuilder::new()
        .model_dir(Some(dir.path().to_path_buf()))
        .build();

    // Discover USB devices
    let devices = controller.discover_usb_devices().unwrap();

    if !devices.is_empty() {
        // Start a thread for the first device
        // This should automatically start model monitoring
        let thread_id = controller.start_usb_device_thread(devices[0]).unwrap();

        // Wait for the model to be loaded
        thread::sleep(Duration::from_millis(500));

        // Get the device state
        let state = controller.get_state(thread_id).unwrap();

        // Check if the model was loaded
        assert!(state.model.is_some(), "Model was not loaded automatically");

        // Check the model
        let loaded_model = state.model.unwrap();
        assert_eq!(loaded_model.name, "PoKeys56U");
        assert_eq!(loaded_model.pins.len(), 2);
        assert!(loaded_model.pins.contains_key(&1));
        assert!(loaded_model.pins.contains_key(&2));

        // Update the model
        let mut updated_model = model.clone();
        updated_model.pins.insert(
            3,
            PinModel {
                capabilities: vec!["DigitalInput".to_string(), "DigitalOutput".to_string()],
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
        assert_eq!(loaded_model.name, "PoKeys56U");
        assert_eq!(loaded_model.pins.len(), 3);
        assert!(loaded_model.pins.contains_key(&1));
        assert!(loaded_model.pins.contains_key(&2));
        assert!(loaded_model.pins.contains_key(&3));
    }
}

// This test requires a connected device, so it will be skipped if no device is found
#[test]
fn test_explicit_model_update() {
    // Skip if no device is connected
    if pokeys_lib::enumerate_usb_devices().unwrap_or(0) == 0 {
        println!("No device connected, skipping test");
        return;
    }

    // Create a thread controller
    let mut controller = ThreadControllerBuilder::new().build();

    // Discover USB devices
    let devices = controller.discover_usb_devices().unwrap();

    if !devices.is_empty() {
        // Start a thread for the first device
        let thread_id = controller.start_usb_device_thread(devices[0]).unwrap();

        // Wait for the model to be loaded
        thread::sleep(Duration::from_millis(500));

        // Create a custom model
        let mut model = DeviceModel {
            name: "CustomModel".to_string(),
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

        model.pins.insert(
            2,
            PinModel {
                capabilities: vec!["DigitalInput".to_string(), "AnalogInput".to_string()],
                active: true,
            },
        );

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
        assert_eq!(loaded_model.name, "CustomModel");
        assert_eq!(loaded_model.pins.len(), 2);
        assert!(loaded_model.pins.contains_key(&1));
        assert!(loaded_model.pins.contains_key(&2));
    }
}

// This test requires a connected device, so it will be skipped if no device is found
#[test]
fn test_model_validation_during_operations() {
    // Skip if no device is connected
    if pokeys_lib::enumerate_usb_devices().unwrap_or(0) == 0 {
        println!("No device connected, skipping test");
        return;
    }

    // Create a temporary directory for the test
    let _dir = tempdir().unwrap();

    // Create a model with limited capabilities
    let mut model = DeviceModel {
        name: "LimitedModel".to_string(),
        pins: HashMap::new(),
    };

    // Add pins with specific capabilities
    model.pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string()], // Only input, no output
            active: true,
        },
    );

    model.pins.insert(
        2,
        PinModel {
            capabilities: vec!["DigitalOutput".to_string()], // Only output, no input
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

        // Wait for the model to be loaded
        thread::sleep(Duration::from_millis(500));

        // Update the device model
        controller
            .update_device_model(thread_id, model.clone())
            .unwrap();

        // Wait for the model to be updated
        thread::sleep(Duration::from_millis(500));

        // Try to set pin 1 as output (should fail)
        let result = controller.set_digital_output(thread_id, 1, true);
        assert!(result.is_err(), "Setting pin 1 as output should fail");

        // Try to set pin 2 as output (should succeed)
        let result = controller.set_digital_output(thread_id, 2, true);
        assert!(result.is_ok(), "Setting pin 2 as output should succeed");

        // Try to read pin 1 as input (should succeed)
        let result = controller.get_digital_input(thread_id, 1);
        assert!(result.is_ok(), "Reading pin 1 as input should succeed");

        // Try to read pin 2 as input (should fail)
        let result = controller.get_digital_input(thread_id, 2);
        assert!(result.is_err(), "Reading pin 2 as input should fail");
    }
}

// This test requires a connected device, so it will be skipped if no device is found
#[test]
fn test_copy_default_models() {
    // Skip if no device is connected
    if pokeys_lib::enumerate_usb_devices().unwrap_or(0) == 0 {
        println!("No device connected, skipping test");
        return;
    }

    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a thread controller with the custom model directory
    let mut controller = ThreadControllerBuilder::new()
        .model_dir(Some(dir.path().to_path_buf()))
        .build();

    // Discover USB devices
    let devices = controller.discover_usb_devices().unwrap();

    if !devices.is_empty() {
        // Start a thread for the first device
        let _thread_id = controller.start_usb_device_thread(devices[0]).unwrap();

        // Wait for the model to be loaded
        thread::sleep(Duration::from_millis(500));

        // Check if the default model files were copied
        let default_models = [
            "PoKeys56U.yaml",
            "PoKeys57U.yaml",
            "PoKeys56E.yaml",
            "PoKeys57E.yaml",
        ];

        for model_file in &default_models {
            let file_path = dir.path().join(model_file);
            assert!(
                file_path.exists(),
                "Default model file {model_file} was not copied"
            );
        }
    }
}

// This test requires a connected device, so it will be skipped if no device is found
#[test]
fn test_model_monitoring_stop_start() {
    // Skip if no device is connected
    if pokeys_lib::enumerate_usb_devices().unwrap_or(0) == 0 {
        println!("No device connected, skipping test");
        return;
    }

    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a model
    let mut model = DeviceModel {
        name: "PoKeys56U".to_string(),
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

    // Create a thread controller with the custom model directory
    let mut controller = ThreadControllerBuilder::new()
        .model_dir(Some(dir.path().to_path_buf()))
        .build();

    // Discover USB devices
    let devices = controller.discover_usb_devices().unwrap();

    if !devices.is_empty() {
        // Start a thread for the first device
        let thread_id = controller.start_usb_device_thread(devices[0]).unwrap();

        // Wait for the model to be loaded
        thread::sleep(Duration::from_millis(500));

        // Stop model monitoring
        controller.stop_model_monitoring(thread_id).unwrap();

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

        // Wait for a while
        thread::sleep(Duration::from_millis(500));

        // Get the device state
        let state = controller.get_state(thread_id).unwrap();

        // Check if the model was NOT updated
        let loaded_model = state.model.unwrap();
        assert_eq!(
            loaded_model.pins.len(),
            1,
            "Model was updated even though monitoring was stopped"
        );

        // Start model monitoring again
        controller
            .start_model_monitoring(thread_id, Some(dir.path().to_path_buf()))
            .unwrap();

        // Wait for the model to be loaded
        thread::sleep(Duration::from_millis(500));

        // Get the device state again
        let state = controller.get_state(thread_id).unwrap();

        // Check if the model was updated
        let loaded_model = state.model.unwrap();
        assert_eq!(
            loaded_model.pins.len(),
            2,
            "Model was not updated after restarting monitoring"
        );
    }
}
