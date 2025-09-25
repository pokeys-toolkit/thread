# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2025-09-25

### Added
- **Phase 1: Core Library Alignment**
  - Fixed PWM data structure compatibility with core library v0.8.0
  - Updated PWM method calls to use pin-based operations
  - Added comprehensive PWM testing (8 test cases)
  - Hardware validation with PoKeys57E device

- **Phase 2: Enhanced Core Features**
  - Enhanced PWM support with servo control (180°, 360° position, 360° speed servos)
  - Enhanced I2C features with automatic fragmentation and retry logic
  - uSPIBridge integration with custom pinout and segment mapping
  - Added 10 comprehensive Phase 2 tests

- **Phase 3: API Modernization**
  - Enhanced device model integration with pin capability validation
  - Enhanced error handling with contextual errors and recovery suggestions
  - Performance optimizations with bulk operations (5x performance improvement)
  - Added 10 comprehensive Phase 3 tests

### Changed
- Updated PWM operations to use `pwm_values` instead of `pwm_duty`
- Enhanced error types with recovery suggestions and classification
- Improved device model integration for hardware-aware operations

### Performance
- Bulk operations demonstrate 5x performance improvement over individual operations
- Optimized state synchronization and device communication

### Testing
- 38 total tests passing (8 Phase 1 + 10 Phase 2 + 10 Phase 3 + 10 integration)
- Hardware validation with real PoKeys57E device
- Comprehensive backward compatibility verification

## [0.2.0] - Previous Release

### Added
- Initial threading architecture for multi-device PoKeys applications
- Per-device threads with thread safety
- State synchronization and output tracking
- Basic device operations and management
