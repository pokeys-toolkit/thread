use log::LevelFilter;
use pokeys_lib::models::DeviceModel;
use pokeys_lib::{ServoConfig, USPIBridgeConfig};

/// Commands that can be sent to device threads
#[derive(Debug, Clone)]
pub enum DeviceCommand {
    /// Start the device thread
    Start,
    /// Pause the device thread
    Pause,
    /// Terminate the device thread
    Terminate,
    /// Restart the device thread
    Restart,
    /// Get the current status of the device thread
    GetStatus,
    /// Set a digital output pin
    SetDigitalOutput { pin: u32, value: bool },
    /// Set an analog output
    SetAnalogOutput { pin: u32, value: u32 },
    /// Set PWM duty cycle
    SetPwmDuty { channel: usize, duty: u32 },
    /// Configure a servo
    ConfigureServo { pin: u8, config: ServoConfig },
    /// Set servo angle
    SetServoAngle { pin: u8, angle: f32 },
    /// Set servo speed
    SetServoSpeed { pin: u8, speed: f32 },
    /// Stop servo
    StopServo { pin: u8 },
    /// I2C write operation
    I2cWrite { address: u8, data: Vec<u8> },
    /// I2C read operation
    I2cRead { address: u8, length: u8 },
    /// I2C write then read operation
    I2cWriteRead { address: u8, write_data: Vec<u8>, read_length: u8 },
    /// I2C bus scan
    I2cScan,
    /// Configure uSPIBridge
    ConfigureUSPIBridge { config: USPIBridgeConfig },
    /// Send uSPIBridge command
    USPIBridgeCommand { command: Vec<u8> },
    /// Bulk set digital outputs
    SetDigitalOutputsBulk { pin_states: Vec<(u32, bool)> },
    /// Bulk set PWM duty cycles
    SetPwmDutiesBulk { channel_duties: Vec<(usize, u32)> },
    /// Bulk read analog inputs
    ReadAnalogInputsBulk { pins: Vec<u32> },
    /// Check pin capability
    CheckPinCapability { pin: u8, capability: String },
    /// Validate pin operation
    ValidatePinOperation { pin: u8, operation: String },
    /// Configure an encoder
    ConfigureEncoder {
        encoder_index: u32,
        pin_a: u32,
        pin_b: u32,
        enabled: bool,
        sampling_4x: bool,
    },
    /// Reset a digital counter
    ResetDigitalCounter { pin: u32 },
    /// Set pin function
    SetPinFunction {
        pin: u32,
        pin_function: pokeys_lib::PinFunction,
    },
    /// Custom command with raw parameters
    Custom {
        request_type: u8,
        param1: u8,
        param2: u8,
        param3: u8,
        param4: u8,
    },
    /// Set log level
    SetLogLevel(LevelFilter),
    /// Update device model
    UpdateModel(DeviceModel),
}
