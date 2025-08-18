# PoKeys Thread - Advanced Threading System

Advanced threading architecture for multi-device PoKeys applications. This library provides thread-safe multi-device management with per-device threads, state synchronization, and concurrent operations.

## ✨ Key Features

### Multi-Device Threading
- **Per-Device Threads**: Each device operates in its own dedicated thread
- **Thread Safety**: Safe concurrent access to device resources
- **State Synchronization**: Automatic state management across threads
- **Output State Tracking**: Comprehensive tracking of device output states

### Concurrent Operations
- **Non-Blocking Operations**: Device operations don't block other devices
- **Parallel Processing**: Multiple devices can be controlled simultaneously  
- **Event-Driven Architecture**: Responsive to device state changes
- **Resource Management**: Automatic cleanup and resource management

### Performance Benefits
- **Scalability**: Efficiently handles multiple devices
- **Responsiveness**: Real-time device interaction
- **Isolation**: Device failures don't affect other devices
- **Load Distribution**: Balanced workload across threads

## 🚀 Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
pokeys-thread = { git = "https://github.com/pokeys-toolkit/thread" }
pokeys-lib = { git = "https://github.com/pokeys-toolkit/core" }
```

## 📖 Usage Examples

### Basic Multi-Device Threading
```rust
use pokeys_thread::*;

fn main() -> Result<()> {
    // Create thread controller
    let mut controller = ThreadControllerBuilder::new()
        .default_refresh_interval(100)
        .build();

    // Start device threads
    let devices = controller.discover_usb_devices()?;
    for device_id in devices {
        controller.start_usb_device_thread(device_id)?;
    }

    // Control devices concurrently
    // Each device operates in its own thread
    Ok(())
}
```

### State Observer Pattern
```rust
use pokeys_thread::*;

fn main() -> Result<()> {
    let mut controller = ThreadControllerBuilder::new().build();
    
    // Start device thread
    controller.start_usb_device_thread(12345678)?;
    
    // Observe state changes
    let observer = controller.create_state_observer(12345678)?;
    
    loop {
        if let Some(state) = observer.get_latest_state() {
            println!("Device state updated: {:?}", state);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
```

### Device Operations
```rust
use pokeys_thread::*;

fn main() -> Result<()> {
    let mut controller = ThreadControllerBuilder::new().build();
    controller.start_usb_device_thread(12345678)?;
    
    // Non-blocking device operations
    controller.set_digital_output(12345678, 1, true)?;
    controller.set_pwm_duty_cycle(12345678, 0, 50.0)?;
    
    // Read device state
    let analog_value = controller.read_analog_input(12345678, 1)?;
    println!("Analog input 1: {}", analog_value);
    
    Ok(())
}
```

## 🏗️ Architecture

### Thread Controller
The `ThreadController` is the main entry point for managing device threads:

- **Device Discovery**: Automatic USB and network device discovery
- **Thread Management**: Start, stop, and monitor device threads
- **Operation Dispatch**: Route operations to appropriate device threads
- **State Management**: Centralized state tracking and synchronization

### Device Threads
Each device runs in its own dedicated thread:

- **Isolated Execution**: Device operations don't interfere with each other
- **Automatic Refresh**: Periodic device state updates
- **Error Handling**: Per-device error isolation and recovery
- **Resource Cleanup**: Automatic resource management on thread termination

### State Synchronization
Thread-safe state management:

- **Atomic Operations**: Lock-free state updates where possible
- **Consistent Views**: Guaranteed consistent state snapshots
- **Change Notifications**: Event-driven state change notifications
- **Output Tracking**: Comprehensive output state tracking

## 🔧 Configuration

### Thread Controller Builder
```rust
let controller = ThreadControllerBuilder::new()
    .default_refresh_interval(50)  // 50ms refresh rate
    .max_retry_attempts(3)         // Retry failed operations
    .timeout_duration(5000)        // 5 second timeout
    .enable_logging(true)          // Enable debug logging
    .build();
```

### Per-Device Configuration
```rust
controller.configure_device_thread(device_id, |config| {
    config
        .refresh_interval(25)      // 25ms for high-speed device
        .priority_mode(true)       // High priority thread
        .buffer_size(1024)         // Larger communication buffer
});
```

## 📚 Examples

The `examples/` directory contains comprehensive examples:

```bash
# Simple controller setup
cargo run --example simple_controller

# State observation patterns
cargo run --example state_observer

# Device operation examples
cargo run --example device_operations

# Logging and debugging
cargo run --example logging_example

# Comprehensive multi-device example
cargo run --example comprehensive_example
```

## 🛡️ Safety & Reliability

### Thread Safety
- **Lock-Free Operations**: Minimal locking for maximum performance
- **Deadlock Prevention**: Careful lock ordering and timeout mechanisms
- **Resource Protection**: Protected access to shared device resources

### Error Handling
- **Isolated Failures**: Device errors don't propagate to other devices
- **Automatic Recovery**: Built-in retry and recovery mechanisms
- **Graceful Degradation**: System continues operating with failed devices

### Performance
- **Minimal Overhead**: Efficient thread management and communication
- **Scalable Architecture**: Handles dozens of devices efficiently
- **Memory Efficient**: Careful memory management and cleanup

## 🧪 Testing

Run the test suite:

```bash
# Unit tests
cargo test

# Integration tests (requires hardware)
cargo test --features hardware-tests

# Performance benchmarks
cargo test --release --features benchmarks
```

## 🤝 Contributing

We welcome contributions! Please ensure:

- All tests pass
- Code follows Rust conventions
- Thread safety is maintained
- Documentation is updated

## 📄 License

This project is licensed under the LGPL-2.1 License - see the [LICENSE](LICENSE) file for details.

## 🔗 Related Projects

- [PoKeys Core](https://github.com/pokeys-toolkit/core) - Core library
- [PoKeys CLI](https://github.com/pokeys-toolkit/cli) - Command-line interface  
- [PoKeys Model Manager](https://github.com/pokeys-toolkit/model-manager) - Device model management

---

**Perfect for**: Industrial automation, multi-device control systems, responsive GUI applications, and any scenario requiring concurrent PoKeys device management.
