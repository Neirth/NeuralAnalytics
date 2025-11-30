use brainflow::{
    board_shim::BoardShim, brainflow_input_params::BrainFlowInputParamsBuilder, BoardIds,
    BrainFlowPresets,
};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::env;
use std::sync::RwLock;

use crate::domain::{models::eeg_work_modes::WorkMode, ports::input::eeg_headset::EegHeadsetPort};

// Default MAC address if environment variable is not set
const DEFAULT_DEVICE_MAC: &str = "C8:8F:B6:6D:E1:E2";

pub struct BrainFlowAdapter {
    board: BoardShim,
    work_mode: WorkMode,
    min_values: RwLock<HashMap<String, f32>>,
    max_values: RwLock<HashMap<String, f32>>,
    has_received_data: RwLock<bool>,  // Track if we've ever received data this session
}

impl Default for BrainFlowAdapter {
    fn default() -> Self {
        let mac_address = env::var("BRAINBIT_MAC_ADDRESS").unwrap_or_else(|_| {
            info!(
                "BRAINBIT_MAC_ADDRESS not set, using default: {}",
                DEFAULT_DEVICE_MAC
            );
            DEFAULT_DEVICE_MAC.to_string()
        });

        debug!("Using MAC Address: {}", mac_address);
        warn!("New instance of BrainFlowAdapter created, check if the device is connected.");

        let params = BrainFlowInputParamsBuilder::default()
            .mac_address(mac_address)
            .timeout(20)
            .build();

        let board_id = BoardIds::BrainbitBoard;
        let board = BoardShim::new(board_id, params).expect("BoardShim initialization failed");

        Self {
            board,
            work_mode: WorkMode::Initialized,
            min_values: RwLock::new(HashMap::new()),
            max_values: RwLock::new(HashMap::new()),
            has_received_data: RwLock::new(false),
        }
    }
}

impl BrainFlowAdapter {
    /// Sends a configuration command to the board and handles the result.
    fn _send_board_command(&self, command: &str) -> Result<String, String> {
        // Stabilize the device before sending commands
        std::thread::sleep(std::time::Duration::from_millis(500));

        debug!("Sending command to board: {}", command);

        // Send the command to the board
        match self.board.config_board(command) {
            Ok(response) => {
                debug!("Command '{}' successful. Response: {}", command, response);
                Ok(response)
            }
            Err(e) => {
                let error_msg = format!("Error sending command '{}': {}", command, e);
                error!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    /// Applies Min-Max scaling to a data series
    ///
    /// This function normalizes the input values according to the observed original range
    /// using the standard Min-Max scaling formula.
    fn _apply_min_max_scaling(&self, data: &[f32], min_orig: f32, max_orig: f32) -> Vec<f32> {
        // Avoid division by zero
        let range_orig = if (max_orig - min_orig).abs() < f32::EPSILON {
            1.0
        } else {
            max_orig - min_orig
        };

        // Apply Min-Max normalization
        data.iter().map(|&v| (v - min_orig) / range_orig).collect()
    }
}

impl EegHeadsetPort for BrainFlowAdapter {
    fn extract_impedance_data(&self) -> Result<HashMap<String, u16>, String> {
        if !matches!(self.work_mode, WorkMode::Calibration) {
            return Err("Device not in Calibration mode. Call change_work_mode first.".to_string());
        }

        // --- IMPORTANT: Define Resistance Channel Indices for BrainBit (PLACEHOLDERS) ---
        // These indices MUST correspond to the ROWS returned by get_board_data()
        // WHEN THE DEVICE IS IN IMPEDANCE MODE.
        // Find these values in the BrainFlow documentation for BrainBitBoard data format.
        const T3_RESISTANCE_IDX: usize = 5; // EXAMPLE - Replace with actual index
        const T4_RESISTANCE_IDX: usize = 6; // EXAMPLE - Replace with actual index
        const O1_RESISTANCE_IDX: usize = 7; // EXAMPLE - Replace with actual index
        const O2_RESISTANCE_IDX: usize = 8; // EXAMPLE - Replace with actual index

        // Map electrode names to their specific RESISTANCE channel indices
        let electrode_channel_map: HashMap<&str, usize> = [
            ("T3", T3_RESISTANCE_IDX),
            ("T4", T4_RESISTANCE_IDX),
            ("O1", O1_RESISTANCE_IDX),
            ("O2", O2_RESISTANCE_IDX),
        ]
        .iter()
        .cloned()
        .collect();
        // --- End Resistance Channel Definition ---

        // Await for the device to stabilize
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Send the command to get impedance data
        let data = self
            .board
            .get_board_data(Some(62), BrainFlowPresets::DefaultPreset)
            .map_err(|e| format!("Failed to get board data for impedance: {}", e))?;

        let mut impedance_values = HashMap::new();

        if data.shape()[0] == 0 {
            return Err("No data returned from board for impedance check.".to_string());
        }

        for (electrode_name, &channel_index) in electrode_channel_map.iter() {
            if channel_index < data.shape()[0] {
                let impedance = if data.row(channel_index).len() > 0 {
                    (data.row(channel_index)[0].abs() / 1000.0) as u16
                } else {
                    0
                };
                impedance_values.insert(electrode_name.to_string(), impedance);
            } else {
                warn!(
                    "Resistance channel index {} for {} out of bounds (rows: {})",
                    channel_index,
                    electrode_name,
                    data.shape()[0]
                );

                impedance_values.insert(electrode_name.to_string(), 0);
            }
        }

        Ok(impedance_values)
    }

    fn extract_raw_data(&self) -> Result<HashMap<String, Vec<f32>>, String> {
        if !matches!(self.work_mode, WorkMode::Extraction) {
            return Err("Device not in Extraction mode. Call change_work_mode first.".to_string());
        }

        // --- IMPORTANT: Define EEG Channel Indices and Names for BrainBit (PLACEHOLDERS) ---
        // These indices MUST correspond to the ROWS returned by get_board_data()
        // WHEN THE DEVICE IS IN SIGNAL EXTRACTION MODE.
        // Find these values in the BrainFlow documentation for BrainBitBoard data format.
        const T3_EEG_IDX: usize = 1; // EXAMPLE - Replace with actual index
        const T4_EEG_IDX: usize = 2; // EXAMPLE - Replace with actual index
        const O1_EEG_IDX: usize = 3; // EXAMPLE - Replace with actual index
        const O2_EEG_IDX: usize = 4; // EXAMPLE - Replace with actual index

        // Map the specific EEG channel indices to their corresponding names
        let channel_map: HashMap<usize, String> = [
            (T3_EEG_IDX, "T3".to_string()),
            (T4_EEG_IDX, "T4".to_string()),
            (O1_EEG_IDX, "O1".to_string()),
            (O2_EEG_IDX, "O2".to_string()),
        ]
        .iter()
        .cloned()
        .collect();
        // --- End EEG Channel Definition ---

        // Await for the device to stabilize
        std::thread::sleep(std::time::Duration::from_millis(300));

        // Check how many samples are available in the buffer
        let available_samples = self
            .board
            .get_board_data_count(BrainFlowPresets::DefaultPreset)
            .unwrap_or(0);
        
        info!(
            "Buffer status: {} samples available, requesting 62",
            available_samples
        );

        // If not enough samples, wait for more data to arrive
        if available_samples < 62 {
            warn!(
                "Not enough samples in buffer ({} < 62). Waiting for more data...",
                available_samples
            );
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            // Check again after waiting
            let available_after_wait = self
                .board
                .get_board_data_count(BrainFlowPresets::DefaultPreset)
                .unwrap_or(0);
            
            // Check if we've ever received data this session
            let has_data_before = *self.has_received_data.read().unwrap();
            
            // Only treat as disconnection if:
            // 1. We had data before (not startup)
            // 2. After waiting 500ms there's STILL 0 samples (device not producing data)
            if available_after_wait == 0 && has_data_before {
                error!("Still no samples after waiting 500ms - device disconnected mid-session!");
                return Err("Device disconnected: no samples after waiting".to_string());
            }
            
            info!("After waiting: {} samples available", available_after_wait);
        }

        // Send the command to get generalist data (this REMOVES data from buffer)
        let data = self
            .board
            .get_board_data(Some(62), BrainFlowPresets::DefaultPreset)
            .map_err(|e| format!("Failed to get board data for raw extraction: {}", e))?;
        
        info!(
            "Retrieved data shape: {} rows x {} columns",
            data.shape()[0],
            data.shape()[1]
        );

        let mut raw_data_map = HashMap::new();

        if data.shape()[0] == 0 {
            warn!("No new raw data returned from get_board_data.");
            return Ok(raw_data_map);
        }

        // Mark that we've successfully received data this session
        {
            let mut has_data = self.has_received_data.write().unwrap();
            if !*has_data {
                info!("First successful data received - session is now active");
                *has_data = true;
            }
        }

        for (&channel_index, channel_name) in channel_map.iter() {
            if channel_index < data.shape()[0] {
                let channel_data_f64 = data.row(channel_index);
                let channel_data_f32: Vec<f32> =
                    channel_data_f64.iter().map(|&v| v as f32).collect();

                // Update min values with RwLock
                {
                    let mut min_values = self.min_values.write().unwrap();
                    if let Some(min_val) = channel_data_f32
                        .iter()
                        .cloned()
                        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    {
                        let current_min = min_values.entry(channel_name.clone()).or_insert(min_val);
                        if min_val < *current_min {
                            *current_min = min_val;
                        }
                    }
                }

                // Update max values with RwLock
                {
                    let mut max_values = self.max_values.write().unwrap();
                    if let Some(max_val) = channel_data_f32
                        .iter()
                        .cloned()
                        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    {
                        let current_max = max_values.entry(channel_name.clone()).or_insert(max_val);
                        if max_val > *current_max {
                            *current_max = max_val;
                        }
                    }
                }

                // Pass raw data directly - the model has BatchNorm and handles normalization internally
                info!(
                    "Channel {} raw data: {} samples, range [{:.2}, {:.2}]",
                    channel_name,
                    channel_data_f32.len(),
                    channel_data_f32.iter().cloned().fold(f32::INFINITY, f32::min),
                    channel_data_f32.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
                );
                raw_data_map.insert(channel_name.clone(), channel_data_f32);
            } else {
                error!(
                    "EEG Channel index {} ('{}') out of bounds for data rows {}",
                    channel_index,
                    channel_name,
                    data.shape()[0]
                );
            }
        }

        Ok(raw_data_map)
    }

    fn change_work_mode(&mut self, new_mode: WorkMode) {
        // Avoid changing if already in the desired mode
        if self.work_mode == new_mode {
            debug!("Already in {:?} mode.", new_mode);
            return; // Or return Ok(()) if the function returns Result
        }

        debug!(
            "Attempting to change work mode from {:?} to {:?}",
            self.work_mode, new_mode
        );

        // 1. Send STOP command for the CURRENT mode
        let stop_command = match self.work_mode {
            WorkMode::Calibration => "CommandStopSignal",
            WorkMode::Extraction => "CommandStopResist",
            WorkMode::Initialized => "CommandStopSignal",
        };

        // Use the private helper function. Abort if stop command fails.
        if self._send_board_command(stop_command).is_err() {
            error!("Mode change aborted due to error stopping current mode.");
            return;
        }

        // 2. Send START command for the NEW mode
        let start_command = match new_mode {
            WorkMode::Calibration => "CommandStartResist",
            WorkMode::Extraction => "CommandStartSignal",
            WorkMode::Initialized => "CommandStartSignal",
        };

        // Use the private helper function. Update state only on success.
        if self._send_board_command(start_command).is_ok() {
            debug!("Successfully changed adapter state to {:?}", new_mode);
            self.work_mode = new_mode;
        } else {
            error!(
                "Mode change failed. Adapter state remains {:?}.",
                self.work_mode
            );
            // State remains unchanged
        }
    }

    /// Connects to the BrainBit device and prepares the session.
    /// If a connection is already established, it returns Ok without any changes.
    fn connect(&self) -> Result<(), String> {
        // Check if the device is already connected
        if self.board.is_prepared().unwrap_or(false) {
            debug!("Device is already connected, ignoring connection request.");
            return Ok(());
        }

        // Attempt to connect to the device
        info!("Attempting to connect to BrainBit device...");

        // Prepare the session with the specified parameters
        let _ = self.board.prepare_session().map_err(|e| {
            let error_msg = format!("Failed to prepare session: {}", e);
            error!("{}", error_msg);
            error_msg
        });

        // Start the stream with a buffer size of 62 and no additional parameters
        let _ = self.board.start_stream(62, "").map_err(|e| {
            let error_msg = format!("Failed to start stream: {}", e);
            error!("{}", error_msg);
            error_msg
        })?;

        if self._send_board_command("CommandStartSignal").is_ok() {
            // Send a log message indicating successful connection
            info!("Connection to BrainBit device established successfully.");
            Ok(())
        } else {
            return Err("Failed to start signal command.".to_string());
        }
    }

    /// Checks if the BrainBit device is connected.
    fn is_connected(&self) -> bool {
        // Check if the device is prepared
        if !self.board.is_prepared().unwrap_or(false) {
            return false;
        } else {
            return true;
        }

        // // Retreive dummy data to check if the device is sending data
        // let _ = self
        //     .board
        //     .get_board_data(Some(1), BrainFlowPresets::DefaultPreset);

        // // Stabilize the device before checking connection
        // std::thread::sleep(std::time::Duration::from_millis(500));

        // // Try to get data from board to check if it's sending data
        // match self
        //     .board
        //     .get_board_data(Some(1), BrainFlowPresets::DefaultPreset)
        // {
        //     Ok(data) => data.shape()[1] != 0,
        //     Err(e) => {
        //         debug!("Error trying to verify the connection of the device: {}", e);
        //         false
        //     }
        // }
    }

    /// Disconnects from the BrainBit device and releases the session.
    fn disconnect(&mut self) -> Result<(), String> {
        if !self.board.is_prepared().unwrap_or(false) {
            return Err("Device is not connected.".to_string());
        }

        // Stop the stream and release the session
        self.board.stop_stream().map_err(|e| {
            let error_msg = format!("Failed to stop stream: {}", e);
            error!("{}", error_msg);
            error_msg
        })?;

        // Attempt to stop the stream
        self.work_mode = WorkMode::Initialized;
        
        // Reset session tracking
        {
            let mut has_data = self.has_received_data.write().unwrap();
            *has_data = false;
        }

        // Release the session
        self.board.release_session().map_err(|e| {
            let error_msg = format!("Failed to release session: {}", e);
            error!("{}", error_msg);
            error_msg
        })
    }

    // Returns the current work mode of the device
    fn get_work_mode(&self) -> WorkMode {
        self.work_mode
    }
}

// Ensure the board is stopped and released when the adapter is dropped
impl Drop for BrainFlowAdapter {
    fn drop(&mut self) {
        debug!("Dropping BrainFlowAdapter, releasing session...");
        if self.board.is_prepared().unwrap_or(false) {
            let _ = self.board.stop_stream(); // Ignore error on stop
            if let Err(e) = self.board.release_session() {
                error!("Error releasing BrainFlow session: {}", e);
            }
        }
    }
}
