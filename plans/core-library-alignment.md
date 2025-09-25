# PoKeys Thread Library - Core Library Alignment Plan

## Overview
Align the PoKeys Thread library (v0.3.0) with the latest PoKeys Core library (v0.18.0) to resolve compilation errors and leverage new features.

## Current Issues Identified

### 1. PWM Data Structure Changes
**Problem**: Core library changed `PwmData` structure from `pwm_duty` field to `pwm_values` array.

**Files Affected**:
- `src/state.rs` (lines 208, 212, 422, 424, 570, 571)

**Required Changes**:
- Replace `pwm.pwm_duty` with `pwm.pwm_values`
- Update array indexing and length checks
- Ensure compatibility with new 6-channel PWM array structure

### 2. PWM Method Name Changes
**Problem**: Core library renamed PWM methods for clarity.

**Files Affected**:
- `src/worker.rs` (line 370)

**Required Changes**:
- Replace `set_pwm_duty_cycle()` with `set_pwm_duty_cycle_for_pin()`
- Update method signatures to match new API

## Implementation Plan

### Phase 1: Fix Compilation Errors (Priority: Critical)

#### Task 1.1: Update PWM Data Structure Usage
- **File**: `src/state.rs`
- **Changes**:
  - Line 208: `self.pwm.pwm_duty.len()` → `self.pwm.pwm_values.len()`
  - Line 212: `self.pwm.pwm_duty[channel]` → `self.pwm.pwm_values[channel]`
  - Line 422: `.pwm_duty` → `.pwm_values`
  - Line 424: `new_pwm.pwm_duty.iter()` → `new_pwm.pwm_values.iter()`
  - Line 570: `state.pwm.pwm_duty.len()` → `state.pwm.pwm_values.len()`
  - Line 571: `state.pwm.pwm_duty[channel]` → `state.pwm.pwm_values[channel]`

#### Task 1.2: Update PWM Method Calls
- **File**: `src/worker.rs`
- **Changes**:
  - Line 370: `device.set_pwm_duty_cycle(channel, duty)` → `device.set_pwm_duty_cycle_for_pin(pin, duty)`
  - Note: May need to convert channel index to pin number (17-22)

### Phase 2: Leverage New Core Features (Priority: Medium)

#### Task 2.1: Enhanced PWM Support
- **Opportunity**: Core library now supports 25MHz PWM with servo control
- **Implementation**:
  - Add servo control wrapper methods to `DeviceOperations` trait
  - Integrate `ServoConfig` and `ServoType` from core library
  - Update examples to demonstrate servo control capabilities

#### Task 2.2: Enhanced I2C Features
- **Opportunity**: Core library has improved I2C with automatic fragmentation and retry logic
- **Implementation**:
  - Update I2C operations to use new enhanced features
  - Add I2C metrics tracking to device state
  - Implement validation level configuration

#### Task 2.3: uSPIBridge Integration
- **Opportunity**: Core library added uSPIBridge support with custom pinout
- **Implementation**:
  - Add uSPIBridge operations to device operations trait
  - Integrate segment mapping functionality
  - Update examples with uSPIBridge usage

### Phase 3: API Modernization (Priority: Low)

#### Task 3.1: Update Device Model Integration
- **Opportunity**: Core library has enhanced device model system
- **Implementation**:
  - Update device initialization to use new model loading
  - Add pin capability validation using device models
  - Implement safety checks for hardware constraints

#### Task 3.2: Enhanced Error Handling
- **Opportunity**: Core library has improved error types with context
- **Implementation**:
  - Update error propagation to use new error types
  - Add recovery suggestions to error handling
  - Implement intelligent retry mechanisms

#### Task 3.3: Performance Optimizations
- **Opportunity**: Core library has bulk operations and optimizations
- **Implementation**:
  - Integrate bulk pin configuration operations
  - Use optimized device enumeration
  - Implement efficient state synchronization

## Testing Strategy

### Unit Tests
- Update existing PWM tests for new data structure
- Add tests for new servo control functionality
- Test I2C enhanced features and error handling

### Integration Tests
- Test thread safety with new core library features
- Validate device model integration
- Test multi-device scenarios with new optimizations

### Hardware Tests
- Test PWM servo control on actual hardware
- Validate I2C improvements with real devices
- Test uSPIBridge functionality if hardware available

## Migration Timeline

### Week 1: Critical Fixes
- [ ] Fix PWM data structure usage (Task 1.1)
- [ ] Fix PWM method calls (Task 1.2)
- [ ] Ensure compilation success
- [ ] Run basic tests

### Week 2: Feature Integration
- [ ] Implement servo control support (Task 2.1)
- [ ] Integrate enhanced I2C features (Task 2.2)
- [ ] Add uSPIBridge support (Task 2.3)

### Week 3: Modernization
- [ ] Update device model integration (Task 3.1)
- [ ] Enhance error handling (Task 3.2)
- [ ] Implement performance optimizations (Task 3.3)

### Week 4: Testing & Documentation
- [ ] Complete test suite updates
- [ ] Update documentation and examples
- [ ] Performance benchmarking
- [ ] Release preparation

## Risk Assessment

### High Risk
- **PWM API Changes**: Breaking changes require careful migration
- **Thread Safety**: New features must maintain thread safety guarantees

### Medium Risk
- **Performance Impact**: New features might affect threading performance
- **Backward Compatibility**: Changes might break existing applications

### Low Risk
- **Documentation**: Updates needed but non-breaking
- **Examples**: May need updates but isolated impact

## Success Criteria

1. **Compilation Success**: All code compiles without errors or warnings
2. **Test Pass Rate**: All existing tests pass with new core library
3. **Feature Parity**: All existing functionality preserved
4. **Performance**: No regression in threading performance
5. **Documentation**: Updated examples and documentation
6. **New Features**: Successfully integrated servo control and enhanced I2C

## Dependencies

- PoKeys Core Library v0.18.0
- Rust toolchain compatibility
- Hardware availability for testing (optional but recommended)

## Notes

- Maintain backward compatibility where possible
- Document all breaking changes clearly
- Consider deprecation warnings for removed functionality
- Ensure thread safety is preserved throughout migration
