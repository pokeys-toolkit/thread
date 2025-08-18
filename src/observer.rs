//! State observer for monitoring state changes

use crate::state::{SharedDeviceState, StateChangeType};
use crossbeam_channel::{Receiver, RecvTimeoutError, TryRecvError};
use log::warn;
use std::sync::Arc;
use std::time::Duration;

/// State observer for monitoring state changes
pub struct StateObserver {
    /// Shared device state
    shared_state: Arc<SharedDeviceState>,
    /// State change notification receiver
    notification_rx: Receiver<StateChangeType>,
    /// Thread ID
    thread_id: u32,
}

impl StateObserver {
    /// Create a new state observer
    pub fn new(thread_id: u32, shared_state: Arc<SharedDeviceState>) -> Self {
        let notification_rx = shared_state.setup_notifications();
        Self {
            shared_state,
            notification_rx,
            thread_id,
        }
    }

    /// Wait for a state change with timeout
    pub fn wait_for_change(&self, timeout: Duration) -> Option<StateChangeType> {
        match self.notification_rx.recv_timeout(timeout) {
            Ok(change_type) => Some(change_type),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => {
                warn!("State observer for thread {} disconnected", self.thread_id);
                None
            }
        }
    }

    /// Check for a state change without blocking
    pub fn check_for_change(&self) -> Option<StateChangeType> {
        match self.notification_rx.try_recv() {
            Ok(change_type) => Some(change_type),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                warn!("State observer for thread {} disconnected", self.thread_id);
                None
            }
        }
    }

    /// Process all pending state changes
    pub fn process_all_changes<F>(&self, mut handler: F)
    where
        F: FnMut(StateChangeType),
    {
        while let Some(change_type) = self.check_for_change() {
            handler(change_type);
        }
    }

    /// Get the shared state
    pub fn shared_state(&self) -> Arc<SharedDeviceState> {
        self.shared_state.clone()
    }

    /// Get the thread ID
    pub fn thread_id(&self) -> u32 {
        self.thread_id
    }
}
