//! # Device State Management
//!
//! This module provides types for managing device state and sharing it between threads.
//! The `DeviceState` struct represents the state of a device, while the `SharedDeviceState`
//! struct provides thread-safe access to the state.
//!
//! ## Usage Example
//!
//! ```ignore
//! use pokeys_thread::{ThreadControllerBuilder, ThreadController, DeviceOperations};
//! use std::time::Duration;
//!
//! // Create a thread controller
//! let mut controller = ThreadControllerBuilder::new().build();
//!
//! // Discover USB devices
//! let devices = controller.discover_usb_devices().unwrap();
//!
//! if !devices.is_empty() {
//!     // Start a thread for the first device
//!     let thread_id = controller.start_usb_device_thread(devices[0]).unwrap();
//!
//!     // Get the device state
//!     let state = controller.get_state(thread_id).unwrap();
//!     println!("Device serial number: {}", state.device_data.serial_number);
//!
//!     // Get the shared state for more advanced operations
//!     let shared_state = controller.get_shared_state(thread_id).unwrap();
//!
//!     // Read a digital input
//!     if let Some(value) = shared_state.get_digital_input(1) {
//!         println!("Digital input 1: {}", value);
//!     }
//!
//!     // Create an observer to monitor state changes
//!     let observer = controller.create_observer(thread_id).unwrap();
//!
//!     // Wait for a state change with timeout
//!     if let Some(change) = observer.wait_for_change(Duration::from_secs(1)) {
//!         println!("State change: {:?}", change);
//!     }
//! }
//! ```

use crossbeam_channel::{Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use pokeys_lib::encoders::EncoderData;
use pokeys_lib::io::PinData;
use pokeys_lib::pwm::PwmData;
use pokeys_lib::{DeviceData, DeviceInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Thread status enumeration.
///
/// Represents the current status of a device thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreadStatus {
    /// Thread is not running
    Stopped,
    /// Thread is running
    Running,
    /// Thread is paused
    Paused,
    /// Thread is in error state
    Error,
}

/// Device state that is shared between threads.
///
/// This struct contains all the state information for a device,
/// including device information, pin data, encoder data, and PWM data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceState {
    /// Device information
    pub device_info: DeviceInfo,
    /// Device data
    pub device_data: DeviceData,
    /// Device model
    pub model: Option<pokeys_lib::models::DeviceModel>,
    /// Pin data
    pub pins: Vec<PinData>,
    /// Encoder data
    pub encoders: Vec<EncoderData>,
    /// PWM data
    pub pwm: PwmData,
    /// Last update timestamp
    pub last_update: u64,
    /// Thread status
    pub status: ThreadStatus,
    /// Error message if any
    pub error_message: Option<String>,
    /// Custom state values
    pub custom_values: HashMap<String, String>,
}

impl DeviceState {
    /// Create a new device state.
    ///
    /// # Parameters
    ///
    /// * `device_info` - The device information.
    /// * `device_data` - The device data.
    ///
    /// # Returns
    ///
    /// A new device state.
    pub fn new(device_info: DeviceInfo, device_data: DeviceData) -> Self {
        Self {
            device_info,
            device_data,
            model: None,
            pins: Vec::new(),
            encoders: Vec::new(),
            pwm: PwmData::new(),
            last_update: 0,
            status: ThreadStatus::Stopped,
            error_message: None,
            custom_values: HashMap::new(),
        }
    }

    /// Update the state from a PoKeys device.
    ///
    /// # Parameters
    ///
    /// * `device` - The PoKeys device to update from.
    pub fn update_from_device(&mut self, device: &pokeys_lib::PoKeysDevice) {
        self.device_info = device.info.clone();
        self.device_data = device.device_data.clone();
        self.model = device.model.clone();
        self.pins = device.pins.clone();
        self.encoders = device.encoders.clone();
        self.pwm = device.pwm.clone();
        self.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }

    /// Get a digital input value.
    ///
    /// # Parameters
    ///
    /// * `pin` - The pin number to read.
    ///
    /// # Returns
    ///
    /// The value of the digital input (true for high, false for low),
    /// or None if the pin is invalid.
    pub fn get_digital_input(&self, pin: u32) -> Option<bool> {
        if pin == 0 || pin as usize > self.pins.len() {
            return None;
        }

        let pin_index = (pin - 1) as usize;
        Some(self.pins[pin_index].digital_value_get != 0)
    }

    /// Get an analog input value.
    ///
    /// # Parameters
    ///
    /// * `pin` - The pin number to read.
    ///
    /// # Returns
    ///
    /// The value of the analog input (0-4095 for 12-bit ADC),
    /// or None if the pin is invalid.
    pub fn get_analog_input(&self, pin: u32) -> Option<u32> {
        if pin == 0 || pin as usize > self.pins.len() {
            return None;
        }

        let pin_index = (pin - 1) as usize;
        Some(self.pins[pin_index].analog_value)
    }

    /// Get an encoder value.
    ///
    /// # Parameters
    ///
    /// * `encoder_index` - The encoder index to read.
    ///
    /// # Returns
    ///
    /// The value of the encoder, or None if the encoder index is invalid.
    pub fn get_encoder_value(&self, encoder_index: u32) -> Option<i32> {
        if encoder_index as usize >= self.encoders.len() {
            return None;
        }

        Some(self.encoders[encoder_index as usize].encoder_value)
    }

    /// Get a PWM duty cycle.
    ///
    /// # Parameters
    ///
    /// * `channel` - The PWM channel to read.
    ///
    /// # Returns
    ///
    /// The duty cycle of the PWM channel (0-4095 for 12-bit PWM),
    /// or None if the channel is invalid.
    pub fn get_pwm_duty_cycle(&self, channel: usize) -> Option<u32> {
        if channel >= self.pwm.pwm_values.len() {
            return None;
        }

        Some(self.pwm.pwm_values[channel])
    }
}

/// State change notification type.
///
/// Represents the type of state change that occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateChangeType {
    /// Digital input changed
    DigitalInput { pin: u32, value: bool },
    /// Digital output changed
    DigitalOutput { pin: u32, value: bool },
    /// Analog input changed
    AnalogInput { pin: u32, value: u32 },
    /// Analog output changed
    AnalogOutput { pin: u32, value: u32 },
    /// Encoder value changed
    EncoderValue { index: u32, value: i32 },
    /// PWM duty cycle changed
    PwmDutyCycle { channel: usize, duty: u32 },
    /// Thread status changed
    ThreadStatus { status: ThreadStatus },
    /// Error occurred
    Error { message: Option<String> },
    /// Custom value changed
    CustomValue { key: String, value: String },
    /// Full state update
    FullUpdate,
}

/// Thread-safe device state container.
///
/// This struct provides thread-safe access to device state
/// and allows for state change notifications.
pub struct SharedDeviceState {
    /// Device state
    state: RwLock<DeviceState>,
    /// Is the thread running
    running: AtomicBool,
    /// Is the thread paused
    paused: AtomicBool,
    /// Last update timestamp
    last_update: AtomicU64,
    /// State change notification sender
    notification_tx: Mutex<Option<Sender<StateChangeType>>>,
}

impl SharedDeviceState {
    /// Create a new shared device state.
    ///
    /// # Parameters
    ///
    /// * `device_info` - The device information.
    /// * `device_data` - The device data.
    ///
    /// # Returns
    ///
    /// A new shared device state.
    pub fn new(device_info: DeviceInfo, device_data: DeviceData) -> Self {
        Self {
            state: RwLock::new(DeviceState::new(device_info, device_data)),
            running: AtomicBool::new(false),
            paused: AtomicBool::new(false),
            last_update: AtomicU64::new(0),
            notification_tx: Mutex::new(None),
        }
    }

    /// Set up state change notifications.
    ///
    /// # Returns
    ///
    /// A receiver for state change notifications.
    pub fn setup_notifications(&self) -> Receiver<StateChangeType> {
        let (tx, rx) = crossbeam_channel::unbounded();
        *self.notification_tx.lock() = Some(tx);
        rx
    }

    /// Send a state change notification.
    ///
    /// # Parameters
    ///
    /// * `change_type` - The type of state change.
    fn notify(&self, change_type: StateChangeType) {
        if let Some(tx) = &*self.notification_tx.lock() {
            let _ = tx.send(change_type);
        }
    }

    /// Get the current thread status.
    ///
    /// # Returns
    ///
    /// The current thread status.
    pub fn status(&self) -> ThreadStatus {
        if !self.running.load(Ordering::Relaxed) {
            ThreadStatus::Stopped
        } else if self.paused.load(Ordering::Relaxed) {
            ThreadStatus::Paused
        } else {
            ThreadStatus::Running
        }
    }

    /// Set the thread as running.
    ///
    /// # Parameters
    ///
    /// * `running` - Whether the thread is running.
    pub fn set_running(&self, running: bool) {
        let old_status = self.status();
        self.running.store(running, Ordering::Relaxed);
        let new_status = self.status();
        if old_status != new_status {
            self.notify(StateChangeType::ThreadStatus { status: new_status });
        }
    }

    /// Set the thread as paused.
    ///
    /// # Parameters
    ///
    /// * `paused` - Whether the thread is paused.
    pub fn set_paused(&self, paused: bool) {
        let old_status = self.status();
        self.paused.store(paused, Ordering::Relaxed);
        let new_status = self.status();
        if old_status != new_status {
            self.notify(StateChangeType::ThreadStatus { status: new_status });
        }
    }

    /// Update the device state from a PoKeys device and detect changes.
    ///
    /// # Parameters
    ///
    /// * `device` - The PoKeys device to update from.
    pub fn update_from_device_with_notifications(&self, device: &pokeys_lib::PoKeysDevice) {
        // First, collect the old state for comparison
        let (old_pins, old_encoders, old_pwm) = self.with_state(|state| {
            (
                state.pins.clone(),
                state.encoders.clone(),
                state.pwm.clone(),
            )
        });

        // Update the state
        self.update(|state| {
            state.update_from_device(device);
        });

        // Now detect changes and send notifications
        let new_state = self.with_state(|state| {
            (
                state.pins.clone(),
                state.encoders.clone(),
                state.pwm.clone(),
            )
        });

        let (new_pins, new_encoders, new_pwm) = new_state;

        // Check for digital input changes
        for (i, (old_pin, new_pin)) in old_pins.iter().zip(new_pins.iter()).enumerate() {
            let pin_number = (i + 1) as u32;

            // Digital input changes
            if old_pin.digital_value_get != new_pin.digital_value_get {
                let value = new_pin.digital_value_get != 0;
                self.notify(StateChangeType::DigitalInput {
                    pin: pin_number,
                    value,
                });
            }

            // Digital output changes
            if old_pin.digital_value_set != new_pin.digital_value_set {
                let value = new_pin.digital_value_set != 0;
                self.notify(StateChangeType::DigitalOutput {
                    pin: pin_number,
                    value,
                });
            }

            // Analog input changes
            if old_pin.analog_value != new_pin.analog_value {
                self.notify(StateChangeType::AnalogInput {
                    pin: pin_number,
                    value: new_pin.analog_value,
                });
            }
        }

        // Check for encoder changes
        for (i, (old_encoder, new_encoder)) in
            old_encoders.iter().zip(new_encoders.iter()).enumerate()
        {
            if old_encoder.encoder_value != new_encoder.encoder_value {
                self.notify(StateChangeType::EncoderValue {
                    index: i as u32,
                    value: new_encoder.encoder_value,
                });
            }
        }

        // Check for PWM changes
        for (i, (old_duty, new_duty)) in old_pwm
            .pwm_values
            .iter()
            .zip(new_pwm.pwm_values.iter())
            .enumerate()
        {
            if old_duty != new_duty {
                self.notify(StateChangeType::PwmDutyCycle {
                    channel: i,
                    duty: *new_duty,
                });
            }
        }
    }
    ///
    /// # Parameters
    ///
    /// * `update_fn` - A function that updates the device state.
    pub fn update(&self, update_fn: impl FnOnce(&mut DeviceState)) {
        let mut state = self.state.write();
        update_fn(&mut state);
        self.last_update.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            Ordering::Relaxed,
        );
        self.notify(StateChangeType::FullUpdate);
    }

    /// Read the device state.
    ///
    /// # Parameters
    ///
    /// * `read_fn` - A function that reads the device state.
    ///
    /// # Returns
    ///
    /// The result of the read function.
    pub fn read<T>(&self, read_fn: impl FnOnce(&DeviceState) -> T) -> T {
        let state = self.state.read();
        read_fn(&state)
    }

    /// Get the last update timestamp.
    ///
    /// # Returns
    ///
    /// The last update timestamp.
    pub fn last_update(&self) -> u64 {
        self.last_update.load(Ordering::Relaxed)
    }

    /// Get a digital input value.
    ///
    /// # Parameters
    ///
    /// * `pin` - The pin number to read.
    ///
    /// # Returns
    ///
    /// The value of the digital input (true for high, false for low),
    /// or None if the pin is invalid.
    pub fn get_digital_input(&self, pin: u32) -> Option<bool> {
        self.read(|state| state.get_digital_input(pin))
    }

    /// Get an analog input value.
    ///
    /// # Parameters
    ///
    /// * `pin` - The pin number to read.
    ///
    /// # Returns
    ///
    /// The value of the analog input (0-4095 for 12-bit ADC),
    /// or None if the pin is invalid.
    pub fn get_analog_input(&self, pin: u32) -> Option<u32> {
        self.read(|state| state.get_analog_input(pin))
    }

    /// Get an encoder value.
    ///
    /// # Parameters
    ///
    /// * `encoder_index` - The encoder index to read.
    ///
    /// # Returns
    ///
    /// The value of the encoder, or None if the encoder index is invalid.
    pub fn get_encoder_value(&self, encoder_index: u32) -> Option<i32> {
        self.read(|state| state.get_encoder_value(encoder_index))
    }

    /// Get a PWM duty cycle.
    ///
    /// # Parameters
    ///
    /// * `channel` - The PWM channel to read.
    ///
    /// # Returns
    ///
    /// The duty cycle of the PWM channel (0-4095 for 12-bit PWM),
    /// or None if the channel is invalid.
    pub fn get_pwm_duty_cycle(&self, channel: usize) -> Option<u32> {
        self.read(|state| state.get_pwm_duty_cycle(channel))
    }

    /// Set a digital output value.
    ///
    /// # Parameters
    ///
    /// * `pin` - The pin number to set.
    /// * `value` - The value to set (true for high, false for low).
    pub fn set_digital_output(&self, pin: u32, value: bool) {
        self.update(|state| {
            if pin > 0 && pin as usize <= state.pins.len() {
                let pin_index = (pin - 1) as usize;
                state.pins[pin_index].digital_value_set = if value { 1 } else { 0 };
                self.notify(StateChangeType::DigitalOutput { pin, value });
            }
        });
    }

    /// Set an analog output value.
    ///
    /// # Parameters
    ///
    /// * `pin` - The pin number to set.
    /// * `value` - The value to set (0-4095 for 12-bit DAC).
    pub fn set_analog_output(&self, pin: u32, value: u32) {
        self.update(|state| {
            if pin > 0 && pin as usize <= state.pins.len() {
                let pin_index = (pin - 1) as usize;
                state.pins[pin_index].analog_value = value;
                self.notify(StateChangeType::AnalogOutput { pin, value });
            }
        });
    }

    /// Set a PWM duty cycle.
    ///
    /// # Parameters
    ///
    /// * `channel` - The PWM channel to set.
    /// * `duty` - The duty cycle to set (0-4095 for 12-bit PWM).
    pub fn set_pwm_duty_cycle(&self, channel: usize, duty: u32) {
        self.update(|state| {
            if channel < state.pwm.pwm_values.len() {
                state.pwm.pwm_values[channel] = duty;
                self.notify(StateChangeType::PwmDutyCycle { channel, duty });
            }
        });
    }

    /// Set a custom value.
    ///
    /// # Parameters
    ///
    /// * `key` - The key of the custom value.
    /// * `value` - The value to set.
    pub fn set_custom_value(&self, key: &str, value: &str) {
        self.update(|state| {
            state
                .custom_values
                .insert(key.to_string(), value.to_string());
            self.notify(StateChangeType::CustomValue {
                key: key.to_string(),
                value: value.to_string(),
            });
        });
    }

    /// Get a custom value.
    ///
    /// # Parameters
    ///
    /// * `key` - The key of the custom value.
    ///
    /// # Returns
    ///
    /// The custom value, or None if the key is not found.
    pub fn get_custom_value(&self, key: &str) -> Option<String> {
        self.read(|state| state.custom_values.get(key).cloned())
    }

    /// Set an error message.
    ///
    /// # Parameters
    ///
    /// * `error` - The error message, or None to clear the error.
    pub fn set_error(&self, error: Option<String>) {
        self.update(|state| {
            state.error_message = error.clone();
            self.notify(StateChangeType::Error { message: error });
        });
    }

    /// Get the error message.
    ///
    /// # Returns
    ///
    /// The error message, or None if there is no error.
    pub fn get_error(&self) -> Option<String> {
        self.read(|state| state.error_message.clone())
    }

    /// Access the device state with a function.
    ///
    /// # Parameters
    ///
    /// * `f` - A function that takes a reference to the device state and returns a value.
    ///
    /// # Returns
    ///
    /// The result of the function.
    pub fn with_state<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&DeviceState) -> T,
    {
        let state = self.state.read();
        f(&state)
    }
}
