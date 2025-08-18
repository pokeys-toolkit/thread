# PoKeys Thread Examples

This directory contains comprehensive examples demonstrating the capabilities of the `pokeys-thread` crate. Each example focuses on different aspects of the threading system and device management.

## Running Examples

To run any example:

```bash
# Build all examples
cargo build --examples -p pokeys-thread

# Run a specific example
cargo run --example <example_name> -p pokeys-thread

# Or run the built executable directly
./target/debug/examples/<example_name>
```

## Example Descriptions

### Basic Examples

#### `simple_controller.rs`
**Purpose**: Introduction to the thread controller basics
- Demonstrates basic device discovery (USB and network)
- Shows how to start device threads
- Basic thread status checking
- Simple device state retrieval

**Key Features**:
- Device enumeration
- Thread creation
- Status monitoring
- Clean shutdown

#### `unified_discovery.rs`
**Purpose**: Unified approach to device discovery
- Shows how to discover both USB and network devices
- Implements fallback logic (USB first, then network)
- Demonstrates connection to the first available device
- Good starting point for applications that need to work with any available device

**Key Features**:
- Unified discovery function
- Fallback logic
- Device information display
- Error handling for discovery failures

### Device Operations

#### `device_operations.rs`
**Purpose**: Comprehensive device I/O operations
- Digital input/output operations
- Analog input reading
- PWM control
- Encoder configuration and reading
- Demonstrates all basic device operations

**Key Features**:
- Digital I/O control
- Analog input reading with voltage conversion
- PWM duty cycle control
- Encoder setup and reading
- Operation error handling

#### `device_model_integration.rs`
**Purpose**: Working with device models for safe operations
- Demonstrates device model loading
- Shows pin capability validation
- Safe operations based on device capabilities
- Model monitoring and updates

**Key Features**:
- Device model information display
- Pin capability checking
- Safe pin operations using model validation
- Model monitoring setup
- Capability-based operation selection

### Advanced Threading

#### `multi_device.rs`
**Purpose**: Managing multiple devices simultaneously
- Discovers and connects to multiple devices
- Demonstrates concurrent device operations
- Shows thread management for multiple devices
- Monitoring multiple device states

**Key Features**:
- Multiple device discovery
- Concurrent thread management
- Parallel device operations
- Multi-device status monitoring
- Coordinated cleanup

#### `thread_management.rs`
**Purpose**: Advanced thread lifecycle management
- Demonstrates the new convenience methods
- Shows individual thread control
- Thread health monitoring
- Lifecycle management best practices

**Key Features**:
- `is_thread_running()` convenience method
- `list_active_threads()` functionality
- Individual thread stopping with `stop_thread()`
- Thread health monitoring
- Lifecycle management patterns

#### `output_tracking.rs`
**Purpose**: Demonstrates digital and analog output state tracking
- Shows the new `StateChangeType::DigitalOutput` functionality
- Demonstrates `StateChangeType::AnalogOutput` tracking
- Real-time monitoring of all output state changes
- Integration with the state observer system

**Key Features**:
- Digital output change detection and notification
- Analog output change detection and notification
- PWM duty cycle change tracking
- Real-time output monitoring
- Comprehensive output state management

#### `state_observer.rs`
**Purpose**: Real-time state change monitoring
- Creates state observers for devices
- Monitors various types of state changes
- Demonstrates event-driven programming
- Shows how to react to device state changes

**Key Features**:
- State observer creation
- Real-time change monitoring
- Different change type handling
- Custom value monitoring
- Event-driven patterns

### System Features

#### `logging_example.rs`
**Purpose**: Comprehensive logging configuration
- Shows how to set up logging
- Demonstrates different log levels
- Thread-specific and global logging control
- Custom logger integration

**Key Features**:
- Custom logger setup
- Thread-specific log levels
- Global log level control
- Logging best practices
- Debug information display

#### `error_handling.rs`
**Purpose**: Robust error handling and recovery
- Comprehensive error handling patterns
- Device connection retry logic
- Operation error handling
- Thread health monitoring
- Recovery strategies

**Key Features**:
- Retry logic for device connections
- Graceful error handling for operations
- Thread health monitoring
- Error recovery patterns
- Graceful degradation strategies

#### `comprehensive_example.rs`
**Purpose**: Complete feature demonstration
- Combines all major features
- Shows real-world usage patterns
- Comprehensive error handling
- Production-ready patterns

**Key Features**:
- All threading features
- Complete device operations
- State monitoring
- Logging integration
- Error handling
- Clean shutdown procedures

## Common Patterns

### Device Discovery Pattern
```rust
// Try USB first, fallback to network
let mut thread_id = None;

// USB devices
match controller.discover_usb_devices() {
    Ok(devices) if !devices.is_empty() => {
        thread_id = Some(controller.start_usb_device_thread(devices[0])?);
    }
    _ => {}
}

// Network devices (fallback)
if thread_id.is_none() {
    match controller.discover_network_devices(2000) {
        Ok(devices) if !devices.is_empty() => {
            thread_id = Some(controller.start_network_device_thread(devices[0].clone())?);
        }
        _ => {}
    }
}
```

### Thread Health Monitoring
```rust
// Check if thread is running
if controller.is_thread_running(thread_id)? {
    println!("Thread is healthy");
}

// Get detailed status
match controller.get_status(thread_id)? {
    ThreadStatus::Running => println!("Running normally"),
    ThreadStatus::Stopped => println!("Thread stopped"),
    ThreadStatus::Error => println!("Thread has error"),
    ThreadStatus::Paused => println!("Thread is paused"),
}
```

### State Change Monitoring
```rust
// Monitor all types of state changes
match change_type {
    StateChangeType::DigitalInput { pin, value } => {
        println!("Digital input {} changed to {}", pin, value);
    }
    StateChangeType::DigitalOutput { pin, value } => {
        println!("Digital output {} changed to {}", pin, value);
    }
    StateChangeType::AnalogInput { pin, value } => {
        println!("Analog input {} changed to {}", pin, value);
    }
    StateChangeType::AnalogOutput { pin, value } => {
        println!("Analog output {} changed to {}", pin, value);
    }
    StateChangeType::EncoderValue { index, value } => {
        println!("Encoder {} changed to {}", index, value);
    }
    StateChangeType::PwmDutyCycle { channel, duty } => {
        println!("PWM channel {} duty changed to {}", channel, duty);
    }
    // ... other change types
}
```

### Safe Device Operations
```rust
// Get device state first
let state = controller.get_state(thread_id)?;

// Check device capabilities
if let Some(model) = &state.model {
    // Find pins with specific capabilities
    for (pin_id, pin_def) in &model.pins {
        if pin_def.capabilities.contains(&"digital_output".to_string()) {
            // Safe to use this pin for digital output
            controller.set_digital_output(thread_id, *pin_id as u32, true)?;
        }
    }
}
```

### Error Handling Pattern
```rust
match controller.set_digital_output(thread_id, pin, true) {
    Ok(_) => {
        info!("Operation successful");
    }
    Err(e) => {
        error!("Operation failed: {}", e);
        // Implement fallback or recovery
    }
}
```

## Prerequisites

- A PoKeys device connected via USB or available on the network
- Proper device drivers installed
- Network devices should be on the same network segment

## Troubleshooting

### No Devices Found
- Ensure PoKeys device is connected and powered
- Check USB drivers are installed
- For network devices, ensure they're on the same network
- Check firewall settings for UDP broadcast packets

### Permission Errors
- On Linux, you may need to add udev rules for USB access
- Run with appropriate permissions or add user to dialout group

### Connection Timeouts
- Increase discovery timeout for network devices
- Check network connectivity
- Ensure device is not in use by another application

## Building and Testing

```bash
# Build all examples
cargo build --examples -p pokeys-thread

# Test all examples (runs each briefly)
./test_examples.sh

# Run specific example with logging
RUST_LOG=debug cargo run --example simple_controller -p pokeys-thread
```

## Next Steps

After exploring these examples:

1. Start with `simple_controller.rs` to understand basics
2. Try `unified_discovery.rs` for practical device connection
3. Explore `device_operations.rs` for I/O operations
4. Use `multi_device.rs` if you have multiple devices
5. Study `error_handling.rs` for production-ready code
6. Check `comprehensive_example.rs` for complete patterns

Each example is self-contained and includes comprehensive error handling and logging to help you understand what's happening during execution.
