use log::LevelFilter;
use pokeys_lib::models::DeviceModel;

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
