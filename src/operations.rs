//! # Device Operations
//!
//! This module provides a trait for performing device operations.
//! The `DeviceOperations` trait is implemented by the `ThreadControllerImpl`
//! to provide a high-level interface for device operations.
//!
//! ## Usage Example
//!
//! ```ignore
//! use pokeys_thread::{ThreadControllerBuilder, ThreadController, DeviceOperations};
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
//!     // Set a digital output
//!     controller.set_digital_output(thread_id, 1, true).unwrap();
//!
//!     // Get a digital input
//!     let input = controller.get_digital_input(thread_id, 2).unwrap();
//!     println!("Digital input 2: {}", input);
//!
//!     // Set a PWM duty cycle
//!     controller.set_pwm_duty_cycle_percent(thread_id, 0, 50.0).unwrap();
//!
//!     // Configure an encoder
//!     controller.configure_encoder(thread_id, 0, 1, 2, true, true).unwrap();
//!
//!     // Get an encoder value
//!     let value = controller.get_encoder_value(thread_id, 0).unwrap();
//!     println!("Encoder 0 value: {}", value);
//! }
//! ```

use crate::error::Result;
use pokeys_lib::{PinCapability, ServoConfig, USPIBridgeConfig};

/// Device operations trait for performing device-specific operations.
///
/// This trait provides a high-level interface for performing device operations
/// such as setting digital outputs, reading digital inputs, configuring encoders,
/// and more. It is implemented by the `ThreadControllerImpl` to provide a
/// convenient way to interact with devices.
pub trait DeviceOperations {
    /// Set a digital output pin.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `pin` - The pin number to set.
    /// * `value` - The value to set (true for high, false for low).
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn set_digital_output(&self, thread_id: u32, pin: u32, value: bool) -> Result<()>;

    /// Get a digital input pin.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to get the input from.
    /// * `pin` - The pin number to read.
    ///
    /// # Returns
    ///
    /// The value of the digital input (true for high, false for low).
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the pin is invalid.
    fn get_digital_input(&self, thread_id: u32, pin: u32) -> Result<bool>;

    /// Set an analog output.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `pin` - The pin number to set.
    /// * `value` - The value to set (0-4095 for 12-bit DAC).
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn set_analog_output(&self, thread_id: u32, pin: u32, value: u32) -> Result<()>;

    /// Get an analog input.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to get the input from.
    /// * `pin` - The pin number to read.
    ///
    /// # Returns
    ///
    /// The value of the analog input (0-4095 for 12-bit ADC).
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the pin is invalid.
    fn get_analog_input(&self, thread_id: u32, pin: u32) -> Result<u32>;

    /// Set a PWM duty cycle.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `channel` - The PWM channel to set.
    /// * `duty` - The duty cycle to set (0-4095 for 12-bit PWM).
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn set_pwm_duty_cycle(&self, thread_id: u32, channel: usize, duty: u32) -> Result<()>;

    /// Set a PWM duty cycle as a percentage.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `channel` - The PWM channel to set.
    /// * `duty_percent` - The duty cycle to set as a percentage (0.0-100.0).
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn set_pwm_duty_cycle_percent(
        &self,
        thread_id: u32,
        channel: usize,
        duty_percent: f32,
    ) -> Result<()>;

    /// Configure a servo on a PWM pin.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `pin` - The PWM pin to configure (17-22).
    /// * `servo_config` - The servo configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn configure_servo(&self, thread_id: u32, pin: u8, servo_config: ServoConfig) -> Result<()>;

    /// Set servo angle (for 180° and 360° position servos).
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `pin` - The PWM pin (17-22).
    /// * `angle` - The angle in degrees (0-180 for 180° servos, 0-360 for 360° position servos).
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn set_servo_angle(&self, thread_id: u32, pin: u8, angle: f32) -> Result<()>;

    /// Set servo speed (for 360° speed servos).
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `pin` - The PWM pin (17-22).
    /// * `speed` - The speed (-100.0 to 100.0, where 0 is stop, positive is clockwise).
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn set_servo_speed(&self, thread_id: u32, pin: u8, speed: f32) -> Result<()>;

    /// Stop a servo (set to neutral position).
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `pin` - The PWM pin (17-22).
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn stop_servo(&self, thread_id: u32, pin: u8) -> Result<()>;

    /// Write data to an I2C device.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `address` - The I2C device address.
    /// * `data` - The data to write.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn i2c_write(&self, thread_id: u32, address: u8, data: Vec<u8>) -> Result<()>;

    /// Read data from an I2C device.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `address` - The I2C device address.
    /// * `length` - The number of bytes to read.
    ///
    /// # Returns
    ///
    /// The data read from the device.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn i2c_read(&self, thread_id: u32, address: u8, length: u8) -> Result<Vec<u8>>;

    /// Write then read from an I2C device (combined operation).
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `address` - The I2C device address.
    /// * `write_data` - The data to write first.
    /// * `read_length` - The number of bytes to read after writing.
    ///
    /// # Returns
    ///
    /// The data read from the device.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn i2c_write_read(
        &self,
        thread_id: u32,
        address: u8,
        write_data: Vec<u8>,
        read_length: u8,
    ) -> Result<Vec<u8>>;

    /// Scan for I2C devices on the bus.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    ///
    /// # Returns
    ///
    /// A vector of found I2C device addresses.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn i2c_scan(&self, thread_id: u32) -> Result<Vec<u8>>;

    /// Configure uSPIBridge with custom pinout.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `config` - The uSPIBridge configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn configure_uspibridge(&self, thread_id: u32, config: USPIBridgeConfig) -> Result<()>;

    /// Send uSPIBridge command.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `command` - The uSPIBridge command data.
    ///
    /// # Returns
    ///
    /// The response data from the uSPIBridge.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn uspibridge_command(&self, thread_id: u32, command: Vec<u8>) -> Result<Vec<u8>>;

    /// Check if a pin supports a specific capability.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to check.
    /// * `pin` - The pin number to check.
    /// * `capability` - The capability to check for.
    ///
    /// # Returns
    ///
    /// True if the pin supports the capability, false otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found.
    fn check_pin_capability(
        &self,
        thread_id: u32,
        pin: u8,
        capability: PinCapability,
    ) -> Result<bool>;

    /// Get device model information.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to get model info from.
    ///
    /// # Returns
    ///
    /// Device model name if available.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found.
    fn get_device_model(&self, thread_id: u32) -> Result<Option<String>>;

    /// Validate pin configuration before operation.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to validate.
    /// * `pin` - The pin number to validate.
    /// * `operation` - Description of the operation being attempted.
    ///
    /// # Errors
    ///
    /// Returns an error if the pin cannot perform the operation.
    fn validate_pin_operation(&self, thread_id: u32, pin: u8, operation: &str) -> Result<()>;

    /// Set multiple digital outputs in a single operation.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `pin_states` - Vector of (pin, state) tuples.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn set_digital_outputs_bulk(&self, thread_id: u32, pin_states: Vec<(u32, bool)>) -> Result<()>;

    /// Set multiple PWM duty cycles in a single operation.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `channel_duties` - Vector of (channel, duty) tuples.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn set_pwm_duties_bulk(&self, thread_id: u32, channel_duties: Vec<(usize, u32)>) -> Result<()>;

    /// Read multiple analog inputs in a single operation.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to read from.
    /// * `pins` - Vector of pin numbers to read.
    ///
    /// # Returns
    ///
    /// Vector of analog values corresponding to the requested pins.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found.
    fn read_analog_inputs_bulk(&self, thread_id: u32, pins: Vec<u32>) -> Result<Vec<u32>>;

    /// Get an encoder value.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to get the encoder value from.
    /// * `encoder_index` - The encoder index to read.
    ///
    /// # Returns
    ///
    /// The value of the encoder.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the encoder index is invalid.
    fn get_encoder_value(&self, thread_id: u32, encoder_index: u32) -> Result<i32>;

    /// Configure an encoder.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `encoder_index` - The encoder index to configure.
    /// * `pin_a` - The pin number for encoder input A.
    /// * `pin_b` - The pin number for encoder input B.
    /// * `enabled` - Whether the encoder is enabled.
    /// * `sampling_4x` - Whether to use 4x sampling.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn configure_encoder(
        &self,
        thread_id: u32,
        encoder_index: u32,
        pin_a: u32,
        pin_b: u32,
        enabled: bool,
        sampling_4x: bool,
    ) -> Result<()>;

    /// Reset a digital counter.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `pin` - The pin number of the counter to reset.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn reset_digital_counter(&self, thread_id: u32, pin: u32) -> Result<()>;

    /// Send a custom request.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `request_type` - The request type.
    /// * `param1` - The first parameter.
    /// * `param2` - The second parameter.
    /// * `param3` - The third parameter.
    /// * `param4` - The fourth parameter.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn send_custom_request(
        &self,
        thread_id: u32,
        request_type: u8,
        param1: u8,
        param2: u8,
        param3: u8,
        param4: u8,
    ) -> Result<()>;

    /// Configure a pin function.
    ///
    /// # Parameters
    ///
    /// * `thread_id` - The ID of the thread to send the command to.
    /// * `pin` - The pin number to configure.
    /// * `pin_function` - The function to set for the pin.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread is not found or if the command send fails.
    fn set_pin_function(
        &self,
        thread_id: u32,
        pin: u32,
        pin_function: pokeys_lib::PinFunction,
    ) -> Result<()>;
}
