use crate::commands::DeviceCommand;
use crate::error::{Result, ThreadError};
use crate::logging::ThreadLogger;
use crate::state::{SharedDeviceState, ThreadStatus};
use crate::sync::DeviceSync;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use log::{debug, error, info, warn, LevelFilter};
use pokeys_lib::{
    connect_to_device, connect_to_network_device, NetworkDeviceSummary, PoKeysDevice,
};
use std::convert::TryInto;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Device connection type
#[derive(Debug, Clone)]
pub enum DeviceType {
    /// USB device with device index
    Usb(u32),
    /// Network device with device summary
    Network(NetworkDeviceSummary),
}

/// Device worker that runs in its own thread
pub trait DeviceWorker {
    /// Start the worker thread
    fn start(&mut self) -> Result<()>;

    /// Stop the worker thread
    fn stop(&mut self) -> Result<()>;

    /// Pause the worker thread
    fn pause(&mut self) -> Result<()>;

    /// Resume the worker thread
    fn resume(&mut self) -> Result<()>;

    /// Get the status of the worker thread
    fn status(&self) -> ThreadStatus;

    /// Get the shared state
    fn shared_state(&self) -> Arc<SharedDeviceState>;

    /// Send a command to the worker thread
    fn send_command(&self, command: DeviceCommand) -> Result<()>;

    /// Get the command sender
    fn command_sender(&self) -> &Sender<DeviceCommand>;

    /// Set the log level
    fn set_log_level(&mut self, level: LevelFilter) -> Result<()>;
}

/// Device worker implementation
pub struct DeviceWorkerImpl {
    /// Thread ID
    thread_id: u32,
    /// Thread handle
    thread_handle: Option<JoinHandle<()>>,
    /// Command sender
    command_tx: Sender<DeviceCommand>,
    /// Command receiver
    command_rx: Option<Receiver<DeviceCommand>>,
    /// Shared device state
    shared_state: Arc<SharedDeviceState>,
    /// Refresh interval in milliseconds
    refresh_interval: u64,
    /// Device type for reconnection
    device_type: DeviceType,
    /// Logger
    logger: Option<Arc<ThreadLogger>>,
}

impl DeviceWorkerImpl {
    /// Create a new device worker for USB device
    pub fn new_usb(
        thread_id: u32,
        device: &PoKeysDevice,
        device_index: u32,
        refresh_interval: u64,
    ) -> Result<(Self, Receiver<DeviceCommand>)> {
        Self::new(
            thread_id,
            device,
            DeviceType::Usb(device_index),
            refresh_interval,
        )
    }

    /// Create a new device worker for network device
    pub fn new_network(
        thread_id: u32,
        device: &PoKeysDevice,
        device_summary: NetworkDeviceSummary,
        refresh_interval: u64,
    ) -> Result<(Self, Receiver<DeviceCommand>)> {
        Self::new(
            thread_id,
            device,
            DeviceType::Network(device_summary),
            refresh_interval,
        )
    }

    /// Create a new device worker
    fn new(
        thread_id: u32,
        device: &PoKeysDevice,
        device_type: DeviceType,
        refresh_interval: u64,
    ) -> Result<(Self, Receiver<DeviceCommand>)> {
        let (command_tx, command_rx) = crossbeam_channel::unbounded();

        let shared_state = Arc::new(SharedDeviceState::new(
            device.info.clone(),
            device.device_data.clone(),
        ));

        // Initialize the shared state with the device data
        shared_state.update(|state| {
            state.pins = device.pins.clone();
            state.encoders = device.encoders.clone();
            state.pwm = device.pwm.clone();
        });

        // Create a second receiver for the worker
        let worker_rx = command_rx.clone();

        Ok((
            Self {
                thread_id,
                thread_handle: None,
                command_tx,
                command_rx: Some(worker_rx),
                shared_state,
                refresh_interval,
                device_type,
                logger: None,
            },
            command_rx,
        ))
    }

    /// Set the logger
    pub fn with_logger(mut self, logger: Arc<ThreadLogger>) -> Self {
        self.logger = Some(logger);
        self
    }

    /// Run the worker thread
    fn run_thread(
        thread_id: u32,
        device_type: DeviceType,
        command_rx: Receiver<DeviceCommand>,
        shared_state: Arc<SharedDeviceState>,
        refresh_interval: u64,
        logger: Option<Arc<ThreadLogger>>,
    ) {
        // Use logger if available, otherwise use standard log macros
        let device_description = match &device_type {
            DeviceType::Usb(index) => format!("USB device index {}", index),
            DeviceType::Network(summary) => {
                format!("network device serial {}", summary.serial_number)
            }
        };

        if let Some(logger) = &logger {
            logger.info(&format!(
                "Device thread {thread_id} started for {device_description}"
            ));
        } else {
            info!("Device thread {thread_id} started for {device_description}");
        }

        // Connect to the device
        let mut device = match &device_type {
            DeviceType::Usb(device_index) => match connect_to_device(*device_index) {
                Ok(device) => device,
                Err(e) => {
                    if let Some(logger) = &logger {
                        logger.error(&format!(
                            "Failed to connect to USB device {device_index}: {e}"
                        ));
                    } else {
                        error!("Failed to connect to USB device {device_index}: {e}");
                    }

                    shared_state.update(|state| {
                        state.error_message = Some(format!("Failed to connect to device: {e}"));
                    });
                    shared_state.set_running(false);
                    return;
                }
            },
            DeviceType::Network(device_summary) => {
                match connect_to_network_device(device_summary) {
                    Ok(device) => device,
                    Err(e) => {
                        if let Some(logger) = &logger {
                            logger.error(&format!(
                                "Failed to connect to network device {}: {e}",
                                device_summary.serial_number
                            ));
                        } else {
                            error!(
                                "Failed to connect to network device {}: {e}",
                                device_summary.serial_number
                            );
                        }

                        shared_state.update(|state| {
                            state.error_message = Some(format!("Failed to connect to device: {e}"));
                        });
                        shared_state.set_running(false);
                        return;
                    }
                }
            }
        };

        // Set the thread as running
        shared_state.set_running(true);
        shared_state.set_paused(false);

        // Create a device sync
        let mut device_sync = DeviceSync::new(thread_id, shared_state.clone(), refresh_interval);

        // Initial sync
        if let Err(e) = device_sync.sync(&mut device) {
            if let Some(logger) = &logger {
                logger.error(&format!("Failed to perform initial sync: {}", e));
            } else {
                error!("Failed to perform initial sync: {}", e);
            }

            shared_state.update(|state| {
                state.error_message = Some(format!("Failed to perform initial sync: {}", e));
            });
        }

        // Main loop
        loop {
            // Check for commands
            match command_rx.try_recv() {
                Ok(command) => {
                    if let Some(logger) = &logger {
                        logger.debug(&format!(
                            "Device thread {} received command: {:?}",
                            thread_id, command
                        ));
                    } else {
                        debug!(
                            "Device thread {} received command: {:?}",
                            thread_id, command
                        );
                    }

                    match command {
                        DeviceCommand::Terminate => {
                            if let Some(logger) = &logger {
                                logger.info(&format!("Device thread {} terminating", thread_id));
                            } else {
                                info!("Device thread {} terminating", thread_id);
                            }

                            shared_state.set_running(false);
                            break;
                        }
                        DeviceCommand::Pause => {
                            if let Some(logger) = &logger {
                                logger.info(&format!("Device thread {} paused", thread_id));
                            } else {
                                info!("Device thread {} paused", thread_id);
                            }

                            shared_state.set_paused(true);
                        }
                        DeviceCommand::Start | DeviceCommand::Restart => {
                            if let Some(logger) = &logger {
                                logger.info(&format!(
                                    "Device thread {} started/restarted",
                                    thread_id
                                ));
                            } else {
                                info!("Device thread {} started/restarted", thread_id);
                            }

                            shared_state.set_running(true);
                            shared_state.set_paused(false);
                        }
                        DeviceCommand::GetStatus => {
                            // Just update the status in the shared state
                            if let Some(logger) = &logger {
                                logger.debug(&format!(
                                    "Device thread {} status: {:?}",
                                    thread_id,
                                    shared_state.status()
                                ));
                            } else {
                                debug!(
                                    "Device thread {} status: {:?}",
                                    thread_id,
                                    shared_state.status()
                                );
                            }
                        }
                        DeviceCommand::SetDigitalOutput { pin, value } => {
                            if let Some(logger) = &logger {
                                logger.debug(&format!(
                                    "Setting digital output pin {} to {}",
                                    pin, value
                                ));
                            } else {
                                debug!("Setting digital output pin {} to {}", pin, value);
                            }

                            if let Err(e) = device.set_digital_output(pin, value) {
                                if let Some(logger) = &logger {
                                    logger.error(&format!("Failed to set digital output: {}", e));
                                } else {
                                    error!("Failed to set digital output: {}", e);
                                }

                                shared_state.update(|state| {
                                    state.error_message =
                                        Some(format!("Failed to set digital output: {}", e));
                                });
                            } else {
                                // Update the pin state in the shared state
                                shared_state.set_digital_output(pin, value);
                            }
                        }
                        DeviceCommand::SetAnalogOutput { pin, value } => {
                            if let Some(logger) = &logger {
                                logger.debug(&format!(
                                    "Setting analog output pin {} to {}",
                                    pin, value
                                ));
                            } else {
                                debug!("Setting analog output pin {} to {}", pin, value);
                            }

                            if let Err(e) = device.set_analog_output(pin, value) {
                                if let Some(logger) = &logger {
                                    logger.error(&format!("Failed to set analog output: {}", e));
                                } else {
                                    error!("Failed to set analog output: {}", e);
                                }

                                shared_state.update(|state| {
                                    state.error_message =
                                        Some(format!("Failed to set analog output: {}", e));
                                });
                            } else {
                                // Update the pin state in the shared state
                                shared_state.set_analog_output(pin, value);
                            }
                        }
                        DeviceCommand::SetPwmDuty { channel, duty } => {
                            if let Some(logger) = &logger {
                                logger.debug(&format!(
                                    "Setting PWM channel {} duty to {}",
                                    channel, duty
                                ));
                            } else {
                                debug!("Setting PWM channel {} duty to {}", channel, duty);
                            }

                            // Convert channel (0-5) to pin number (17-22)
                            // PWM channels map: 0->22, 1->21, 2->20, 3->19, 4->18, 5->17
                            let pin = match channel {
                                0 => 22,
                                1 => 21,
                                2 => 20,
                                3 => 19,
                                4 => 18,
                                5 => 17,
                                _ => {
                                    if let Some(logger) = &logger {
                                        logger.error(&format!("Invalid PWM channel: {}", channel));
                                    } else {
                                        error!("Invalid PWM channel: {}", channel);
                                    }
                                    continue;
                                }
                            };

                            if let Err(e) = device.set_pwm_duty_cycle_for_pin(pin, duty) {
                                if let Some(logger) = &logger {
                                    logger.error(&format!("Failed to set PWM duty cycle: {}", e));
                                } else {
                                    error!("Failed to set PWM duty cycle: {}", e);
                                }

                                shared_state.update(|state| {
                                    state.error_message =
                                        Some(format!("Failed to set PWM duty cycle: {}", e));
                                });
                            } else {
                                // Update the PWM state in the shared state
                                shared_state.set_pwm_duty_cycle(channel, duty);
                            }
                        }
                        DeviceCommand::ConfigureServo { pin, config } => {
                            if let Some(logger) = &logger {
                                logger.debug(&format!("Configuring servo on pin {}", pin));
                            } else {
                                debug!("Configuring servo on pin {}", pin);
                            }

                            // Store servo configuration in device state
                            // Note: This would typically be stored in a servo configuration map
                            // For now, we'll just log the configuration
                            if let Some(logger) = &logger {
                                logger.info(&format!("Servo configured on pin {} with config: {:?}", pin, config));
                            } else {
                                info!("Servo configured on pin {} with config: {:?}", pin, config);
                            }
                        }
                        DeviceCommand::SetServoAngle { pin, angle } => {
                            if let Some(logger) = &logger {
                                logger.debug(&format!("Setting servo angle on pin {} to {}", pin, angle));
                            } else {
                                debug!("Setting servo angle on pin {} to {}", pin, angle);
                            }

                            // For now, convert angle to PWM duty cycle (simplified implementation)
                            // In a full implementation, this would use stored servo configuration
                            let duty = ((angle / 180.0) * 4095.0) as u32;
                            
                            if let Err(e) = device.set_pwm_duty_cycle_for_pin(pin, duty) {
                                if let Some(logger) = &logger {
                                    logger.error(&format!("Failed to set servo angle: {}", e));
                                } else {
                                    error!("Failed to set servo angle: {}", e);
                                }
                            }
                        }
                        DeviceCommand::SetServoSpeed { pin, speed } => {
                            if let Some(logger) = &logger {
                                logger.debug(&format!("Setting servo speed on pin {} to {}", pin, speed));
                            } else {
                                debug!("Setting servo speed on pin {} to {}", pin, speed);
                            }

                            // Convert speed (-100 to 100) to PWM duty cycle
                            let duty = (((speed + 100.0) / 200.0) * 4095.0) as u32;
                            
                            if let Err(e) = device.set_pwm_duty_cycle_for_pin(pin, duty) {
                                if let Some(logger) = &logger {
                                    logger.error(&format!("Failed to set servo speed: {}", e));
                                } else {
                                    error!("Failed to set servo speed: {}", e);
                                }
                            }
                        }
                        DeviceCommand::StopServo { pin } => {
                            if let Some(logger) = &logger {
                                logger.debug(&format!("Stopping servo on pin {}", pin));
                            } else {
                                debug!("Stopping servo on pin {}", pin);
                            }

                            // Set to neutral position (1.5ms pulse = ~1500 duty cycle)
                            let duty = 1500;
                            
                            if let Err(e) = device.set_pwm_duty_cycle_for_pin(pin, duty) {
                                if let Some(logger) = &logger {
                                    logger.error(&format!("Failed to stop servo: {}", e));
                                } else {
                                    error!("Failed to stop servo: {}", e);
                                }
                            }
                        }
                        DeviceCommand::I2cWrite { address, data } => {
                            if let Some(logger) = &logger {
                                logger.debug(&format!("I2C write to address 0x{:02X}", address));
                            } else {
                                debug!("I2C write to address 0x{:02X}", address);
                            }

                            if let Err(e) = pokeys_lib::i2c_write_simple(&mut device, address, &data) {
                                if let Some(logger) = &logger {
                                    logger.error(&format!("Failed to write I2C: {}", e));
                                } else {
                                    error!("Failed to write I2C: {}", e);
                                }
                            }
                        }
                        DeviceCommand::I2cRead { address, length } => {
                            if let Some(logger) = &logger {
                                logger.debug(&format!("I2C read from address 0x{:02X}", address));
                            } else {
                                debug!("I2C read from address 0x{:02X}", address);
                            }

                            match pokeys_lib::i2c_read_simple(&mut device, address, length) {
                                Ok(data) => {
                                    if let Some(logger) = &logger {
                                        logger.debug(&format!("I2C read {} bytes", data.len()));
                                    } else {
                                        debug!("I2C read {} bytes", data.len());
                                    }
                                }
                                Err(e) => {
                                    if let Some(logger) = &logger {
                                        logger.error(&format!("Failed to read I2C: {}", e));
                                    } else {
                                        error!("Failed to read I2C: {}", e);
                                    }
                                }
                            }
                        }
                        DeviceCommand::I2cWriteRead { address, write_data, read_length } => {
                            if let Some(logger) = &logger {
                                logger.debug(&format!("I2C write-read to address 0x{:02X}", address));
                            } else {
                                debug!("I2C write-read to address 0x{:02X}", address);
                            }

                            // Perform write then read operation
                            if let Err(e) = pokeys_lib::i2c_write_simple(&mut device, address, &write_data) {
                                if let Some(logger) = &logger {
                                    logger.error(&format!("Failed to write I2C: {}", e));
                                } else {
                                    error!("Failed to write I2C: {}", e);
                                }
                            } else {
                                match pokeys_lib::i2c_read_simple(&mut device, address, read_length) {
                                    Ok(data) => {
                                        if let Some(logger) = &logger {
                                            logger.debug(&format!("I2C read {} bytes", data.len()));
                                        } else {
                                            debug!("I2C read {} bytes", data.len());
                                        }
                                    }
                                    Err(e) => {
                                        if let Some(logger) = &logger {
                                            logger.error(&format!("Failed to read I2C: {}", e));
                                        } else {
                                            error!("Failed to read I2C: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        DeviceCommand::I2cScan => {
                            if let Some(logger) = &logger {
                                logger.debug("Scanning I2C bus");
                            } else {
                                debug!("Scanning I2C bus");
                            }

                            // Scan I2C addresses 0x08 to 0x77
                            for addr in 0x08..=0x77 {
                                if let Ok(_) = pokeys_lib::i2c_read_simple(&mut device, addr, 1) {
                                    if let Some(logger) = &logger {
                                        logger.info(&format!("Found I2C device at address 0x{:02X}", addr));
                                    } else {
                                        info!("Found I2C device at address 0x{:02X}", addr);
                                    }
                                }
                            }
                        }
                        DeviceCommand::ConfigureUSPIBridge { config } => {
                            if let Some(logger) = &logger {
                                logger.debug("Configuring uSPIBridge");
                            } else {
                                debug!("Configuring uSPIBridge");
                            }

                            // For now, just log the configuration
                            // Full implementation would configure the uSPIBridge hardware
                            if let Some(logger) = &logger {
                                logger.info(&format!("uSPIBridge configured with {} devices", config.device_count));
                            } else {
                                info!("uSPIBridge configured with {} devices", config.device_count);
                            }
                        }
                        DeviceCommand::USPIBridgeCommand { command } => {
                            if let Some(logger) = &logger {
                                logger.debug("Sending uSPIBridge command");
                            } else {
                                debug!("Sending uSPIBridge command");
                            }

                            // For now, just log the command
                            // Full implementation would send the command via uSPIBridge
                            if let Some(logger) = &logger {
                                logger.debug(&format!("uSPIBridge command: {} bytes", command.len()));
                            } else {
                                debug!("uSPIBridge command: {} bytes", command.len());
                            }
                        }
                        DeviceCommand::ConfigureEncoder {
                            encoder_index,
                            pin_a,
                            pin_b,
                            enabled,
                            sampling_4x,
                        } => {
                            if let Some(logger) = &logger {
                                logger.debug(&format!(
                                    "Configuring encoder {} on pins {} and {}",
                                    encoder_index, pin_a, pin_b
                                ));
                            } else {
                                debug!(
                                    "Configuring encoder {} on pins {} and {}",
                                    encoder_index, pin_a, pin_b
                                );
                            }

                            let mut options = pokeys_lib::encoders::EncoderOptions::new();
                            options.enabled = enabled;
                            options.sampling_4x = sampling_4x;

                            // Convert u32 to u8 for pin_a and pin_b
                            let pin_a_u8: u8 = match pin_a.try_into() {
                                Ok(val) => val,
                                Err(_) => {
                                    if let Some(logger) = &logger {
                                        logger.error(&format!(
                                            "Pin A value {} is out of range for u8",
                                            pin_a
                                        ));
                                    } else {
                                        error!("Pin A value {} is out of range for u8", pin_a);
                                    }

                                    shared_state.update(|state| {
                                        state.error_message = Some(format!(
                                            "Pin A value {} is out of range for u8",
                                            pin_a
                                        ));
                                    });
                                    continue;
                                }
                            };

                            let pin_b_u8: u8 = match pin_b.try_into() {
                                Ok(val) => val,
                                Err(_) => {
                                    if let Some(logger) = &logger {
                                        logger.error(&format!(
                                            "Pin B value {} is out of range for u8",
                                            pin_b
                                        ));
                                    } else {
                                        error!("Pin B value {} is out of range for u8", pin_b);
                                    }

                                    shared_state.update(|state| {
                                        state.error_message = Some(format!(
                                            "Pin B value {} is out of range for u8",
                                            pin_b
                                        ));
                                    });
                                    continue;
                                }
                            };

                            if let Err(e) = device.configure_encoder(
                                encoder_index as u8,
                                pin_a_u8,
                                pin_b_u8,
                                options,
                            ) {
                                if let Some(logger) = &logger {
                                    logger.error(&format!("Failed to configure encoder: {}", e));
                                } else {
                                    error!("Failed to configure encoder: {}", e);
                                }

                                shared_state.update(|state| {
                                    state.error_message =
                                        Some(format!("Failed to configure encoder: {}", e));
                                });
                            } else {
                                // The encoder state will be updated in the next sync
                            }
                        }
                        DeviceCommand::ResetDigitalCounter { pin } => {
                            if let Some(logger) = &logger {
                                logger.debug(&format!("Resetting digital counter for pin {}", pin));
                            } else {
                                debug!("Resetting digital counter for pin {}", pin);
                            }

                            if let Err(e) = device.reset_digital_counter(pin) {
                                if let Some(logger) = &logger {
                                    logger
                                        .error(&format!("Failed to reset digital counter: {}", e));
                                } else {
                                    error!("Failed to reset digital counter: {}", e);
                                }

                                shared_state.update(|state| {
                                    state.error_message =
                                        Some(format!("Failed to reset digital counter: {}", e));
                                });
                            }
                        }
                        DeviceCommand::Custom {
                            request_type,
                            param1,
                            param2,
                            param3,
                            param4,
                        } => {
                            if let Some(logger) = &logger {
                                logger.debug(&format!(
                                    "Sending custom request: {:02X} {:02X} {:02X} {:02X} {:02X}",
                                    request_type, param1, param2, param3, param4
                                ));
                            } else {
                                debug!(
                                    "Sending custom request: {:02X} {:02X} {:02X} {:02X} {:02X}",
                                    request_type, param1, param2, param3, param4
                                );
                            }

                            if let Err(e) =
                                device.custom_request(request_type, param1, param2, param3, param4)
                            {
                                if let Some(logger) = &logger {
                                    logger.error(&format!("Failed to send custom request: {}", e));
                                } else {
                                    error!("Failed to send custom request: {}", e);
                                }

                                shared_state.update(|state| {
                                    state.error_message =
                                        Some(format!("Failed to send custom request: {}", e));
                                });
                            }
                        }
                        DeviceCommand::SetLogLevel(level) => {
                            if let Some(logger) = &logger {
                                logger.info(&format!("Setting log level to {:?}", level));
                            } else {
                                info!("Setting log level to {:?}", level);
                            }
                            // The actual log level change is handled by the controller
                        }
                        DeviceCommand::SetPinFunction { pin, pin_function } => {
                            if let Some(logger) = &logger {
                                logger.debug(&format!(
                                    "Setting pin {} function to {:?}",
                                    pin, pin_function
                                ));
                            } else {
                                debug!("Setting pin {} function to {:?}", pin, pin_function);
                            }

                            if let Err(e) = device.set_pin_function(pin, pin_function) {
                                if let Some(logger) = &logger {
                                    logger.error(&format!("Failed to set pin function: {}", e));
                                } else {
                                    error!("Failed to set pin function: {}", e);
                                }

                                shared_state.update(|state| {
                                    state.error_message =
                                        Some(format!("Failed to set pin function: {}", e));
                                });
                            } else if let Some(logger) = &logger {
                                logger.info(&format!(
                                    "Successfully configured pin {} as {:?}",
                                    pin, pin_function
                                ));
                            } else {
                                info!("Successfully configured pin {} as {:?}", pin, pin_function);
                            }
                        }
                        DeviceCommand::UpdateModel(model) => {
                            if let Some(logger) = &logger {
                                logger.info(&format!("Updating device model to {}", model.name));
                            } else {
                                info!("Updating device model to {}", model.name);
                            }

                            // Update the model in the device
                            device.model = Some(model.clone());

                            // Update the model in the shared state
                            shared_state.update(|state| {
                                state.model = Some(model);
                            });

                            // Restart the device to apply the new model
                            if let Some(logger) = &logger {
                                logger.info("Restarting device to apply new model");
                            } else {
                                info!("Restarting device to apply new model");
                            }

                            // Temporarily set paused to true to avoid device operations during restart
                            shared_state.set_paused(true);

                            // Reconnect to the device
                            match device_type {
                                DeviceType::Usb(index) => {
                                    match connect_to_device(index) {
                                        Ok(new_device) => {
                                            device = new_device;
                                            // Transfer the model to the new device
                                            device.model = shared_state
                                                .with_state(|state| state.model.clone());

                                            if let Some(logger) = &logger {
                                                logger.info("Device reconnected successfully");
                                            } else {
                                                info!("Device reconnected successfully");
                                            }
                                        }
                                        Err(e) => {
                                            if let Some(logger) = &logger {
                                                logger.error(&format!(
                                                    "Failed to reconnect to device: {}",
                                                    e
                                                ));
                                            } else {
                                                error!("Failed to reconnect to device: {}", e);
                                            }

                                            shared_state.update(|state| {
                                                state.error_message = Some(format!(
                                                    "Failed to reconnect to device: {}",
                                                    e
                                                ));
                                            });
                                        }
                                    }
                                }
                                DeviceType::Network(ref summary) => {
                                    match connect_to_network_device(summary) {
                                        Ok(new_device) => {
                                            device = new_device;
                                            // Transfer the model to the new device
                                            device.model = shared_state
                                                .with_state(|state| state.model.clone());

                                            if let Some(logger) = &logger {
                                                logger.info("Device reconnected successfully");
                                            } else {
                                                info!("Device reconnected successfully");
                                            }
                                        }
                                        Err(e) => {
                                            if let Some(logger) = &logger {
                                                logger.error(&format!(
                                                    "Failed to reconnect to device: {}",
                                                    e
                                                ));
                                            } else {
                                                error!("Failed to reconnect to device: {}", e);
                                            }

                                            shared_state.update(|state| {
                                                state.error_message = Some(format!(
                                                    "Failed to reconnect to device: {}",
                                                    e
                                                ));
                                            });
                                        }
                                    }
                                }
                            }

                            // Resume device operations
                            shared_state.set_paused(false);
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    // No command available, continue
                }
                Err(TryRecvError::Disconnected) => {
                    // Command channel disconnected, terminate thread
                    if let Some(logger) = &logger {
                        logger.warn(&format!(
                            "Device thread {} command channel disconnected, terminating",
                            thread_id
                        ));
                    } else {
                        warn!(
                            "Device thread {} command channel disconnected, terminating",
                            thread_id
                        );
                    }

                    shared_state.set_running(false);
                    break;
                }
            }

            // If paused, skip the sync
            if shared_state.status() == ThreadStatus::Paused {
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            // Check if it's time to sync the device state
            if device_sync.should_sync() {
                if let Err(e) = device_sync.sync(&mut device) {
                    if let Some(logger) = &logger {
                        logger.error(&format!("Failed to sync device state: {}", e));
                    } else {
                        error!("Failed to sync device state: {}", e);
                    }
                    // Continue running even if sync fails
                }
            }

            // Sleep a bit to avoid busy-waiting
            thread::sleep(Duration::from_millis(10));
        }

        if let Some(logger) = &logger {
            logger.info(&format!("Device thread {} terminated", thread_id));
        } else {
            info!("Device thread {} terminated", thread_id);
        }
    }
}

impl DeviceWorker for DeviceWorkerImpl {
    fn start(&mut self) -> Result<()> {
        // Check if the thread is already running
        if self.thread_handle.is_some() {
            return Err(ThreadError::ThreadAlreadyExists(self.thread_id));
        }

        // Clone the necessary data for the thread
        let thread_id = self.thread_id;
        let device_type = self.device_type.clone();
        let command_rx = match self.command_rx.take() {
            Some(rx) => rx,
            None => {
                return Err(ThreadError::CommandSendFailed(
                    "Command receiver already taken".to_string(),
                ))
            }
        };
        let shared_state = self.shared_state.clone();
        let refresh_interval = self.refresh_interval;
        let logger = self.logger.clone();

        // Start the thread
        let handle = thread::spawn(move || {
            Self::run_thread(
                thread_id,
                device_type,
                command_rx,
                shared_state,
                refresh_interval,
                logger,
            );
        });

        self.thread_handle = Some(handle);

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        // Send terminate command
        self.send_command(DeviceCommand::Terminate)?;

        // Wait for the thread to finish
        if let Some(handle) = self.thread_handle.take() {
            handle.join().map_err(|_| ThreadError::ThreadJoinError)?;
        }

        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        self.send_command(DeviceCommand::Pause)
    }

    fn resume(&mut self) -> Result<()> {
        self.send_command(DeviceCommand::Start)
    }

    fn status(&self) -> ThreadStatus {
        self.shared_state.status()
    }

    fn shared_state(&self) -> Arc<SharedDeviceState> {
        self.shared_state.clone()
    }

    fn send_command(&self, command: DeviceCommand) -> Result<()> {
        self.command_tx
            .send(command)
            .map_err(|e| ThreadError::CommandSendFailed(e.to_string()))
    }

    fn command_sender(&self) -> &Sender<DeviceCommand> {
        &self.command_tx
    }

    fn set_log_level(&mut self, level: LevelFilter) -> Result<()> {
        self.send_command(DeviceCommand::SetLogLevel(level))
    }
}
