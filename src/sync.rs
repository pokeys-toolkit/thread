//! Data synchronization

use crate::error::{Result, ThreadError};
use crate::state::SharedDeviceState;
use log::error;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Data synchronization
pub struct DeviceSync {
    /// Shared device state
    shared_state: Arc<SharedDeviceState>,
    /// Thread ID
    thread_id: u32,
    /// Last sync time
    last_sync: Instant,
    /// Sync interval
    sync_interval: Duration,
}

impl DeviceSync {
    /// Create a new data synchronization
    pub fn new(
        thread_id: u32,
        shared_state: Arc<SharedDeviceState>,
        sync_interval_ms: u64,
    ) -> Self {
        Self {
            shared_state,
            thread_id,
            last_sync: Instant::now(),
            sync_interval: Duration::from_millis(sync_interval_ms),
        }
    }

    /// Check if it's time to sync
    pub fn should_sync(&self) -> bool {
        self.last_sync.elapsed() >= self.sync_interval
    }

    /// Sync the device state
    pub fn sync(&mut self, device: &mut pokeys_lib::PoKeysDevice) -> Result<()> {
        // debug!("Syncing device state for thread {}", self.thread_id);

        // Refresh digital inputs
        if let Err(e) = device.get_digital_inputs() {
            error!("Failed to refresh digital inputs: {e}");
            self.shared_state
                .set_error(Some(format!("Failed to refresh digital inputs: {e}")));
            return Err(ThreadError::DeviceError(e));
        }

        // Refresh analog inputs
        if let Err(e) = device.read_analog_inputs() {
            error!("Failed to refresh analog inputs: {e}");
            self.shared_state
                .set_error(Some(format!("Failed to refresh analog inputs: {e}")));
            return Err(ThreadError::DeviceError(e));
        }

        // Refresh encoder values
        for i in 0..device.encoders.len() {
            if let Err(e) = device.get_encoder_value(i as u8) {
                error!("Failed to refresh encoder {i}: {e}");
                self.shared_state
                    .set_error(Some(format!("Failed to refresh encoder {i}: {e}")));
                // Continue with other encoders even if one fails
            }
        }

        // Update the shared state with the refreshed device state and detect changes
        self.shared_state
            .update_from_device_with_notifications(device);

        self.last_sync = Instant::now();
        Ok(())
    }

    /// Get the shared state
    pub fn shared_state(&self) -> Arc<SharedDeviceState> {
        self.shared_state.clone()
    }

    /// Get the thread ID
    pub fn thread_id(&self) -> u32 {
        self.thread_id
    }

    /// Get the sync interval
    pub fn sync_interval(&self) -> Duration {
        self.sync_interval
    }

    /// Set the sync interval
    pub fn set_sync_interval(&mut self, sync_interval_ms: u64) {
        self.sync_interval = Duration::from_millis(sync_interval_ms);
    }
}
