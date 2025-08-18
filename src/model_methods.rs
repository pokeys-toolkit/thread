    /// Start model monitoring for a device thread
    ///
    /// This method starts monitoring the device model file for changes and
    /// updates the device model when changes are detected.
    ///
    /// # Arguments
    ///
    /// * `thread_id` - The ID of the thread to monitor
    /// * `model_dir` - Optional custom directory for model files
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if monitoring started successfully, an error otherwise
    pub fn start_model_monitoring(
        &mut self,
        thread_id: usize,
        model_dir: Option<std::path::PathBuf>,
    ) -> Result<()> {
        // Check if the thread exists
        let worker = self
            .workers
            .get(&thread_id)
            .ok_or_else(|| ThreadError::InvalidThreadId(thread_id))?;

        // Get the device state
        let state = worker.shared_state().with_state(|state| state.clone());

        // Get the device model name
        let model_name = match state.device_data.device_type_id {
            10 => "PoKeys56U", // DeviceTypeId::Device56U
            30 => "PoKeys57U", // DeviceTypeId::Device57U
            31 => "PoKeys57E", // DeviceTypeId::Device57E
            11 => "PoKeys56E", // DeviceTypeId::Device56E
            _ => return Err(ThreadError::UnsupportedDevice),
        };

        // Create a model monitor
        let dir = model_dir.unwrap_or_else(|| {
            let mut path = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            path.push(".config/pokeys/models");
            path
        });

        // Create the directory if it doesn't exist
        if !dir.exists() {
            std::fs::create_dir_all(&dir).map_err(|e| {
                ThreadError::Other(format!("Failed to create model directory: {}", e))
            })?;
        }

        // Create a callback that sends model updates to the device thread
        let worker_clone = worker.clone();
        let callback = move |_: String, model: pokeys_lib::models::DeviceModel| {
            // Send the model update command to the device thread
            let _ = worker_clone.send_command(DeviceCommand::UpdateModel(model));
        };

        // Create and start the model monitor
        let mut monitor = pokeys_lib::models::ModelMonitor::new(dir, callback);
        monitor.start().map_err(|e| {
            ThreadError::Other(format!("Failed to start model monitoring: {}", e))
        })?;

        // Store the monitor
        self.model_monitors.insert(thread_id, monitor);

        Ok(())
    }

    /// Stop model monitoring for a device thread
    ///
    /// # Arguments
    ///
    /// * `thread_id` - The ID of the thread to stop monitoring
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if monitoring stopped successfully, an error otherwise
    pub fn stop_model_monitoring(&mut self, thread_id: usize) -> Result<()> {
        // Check if the thread exists
        if !self.workers.contains_key(&thread_id) {
            return Err(ThreadError::InvalidThreadId(thread_id));
        }

        // Stop and remove the monitor
        if let Some(mut monitor) = self.model_monitors.remove(&thread_id) {
            monitor.stop().map_err(|e| {
                ThreadError::Other(format!("Failed to stop model monitoring: {}", e))
            })?;
        }

        Ok(())
    }

    /// Update the device model
    ///
    /// # Arguments
    ///
    /// * `thread_id` - The ID of the thread to update
    /// * `model` - The new device model
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if the model was updated successfully, an error otherwise
    pub fn update_device_model(
        &self,
        thread_id: usize,
        model: pokeys_lib::models::DeviceModel,
    ) -> Result<()> {
        // Check if the thread exists
        let worker = self
            .workers
            .get(&thread_id)
            .ok_or_else(|| ThreadError::InvalidThreadId(thread_id))?;

        // Send the model update command to the device thread
        worker.send_command(DeviceCommand::UpdateModel(model))?;

        Ok(())
    }
