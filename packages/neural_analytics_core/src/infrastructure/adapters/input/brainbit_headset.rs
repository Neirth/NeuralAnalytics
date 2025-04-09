// adapter.rs
use blackbox_di::{factory, implements, injectable};
use brainflow::{
    board_shim::BoardShim, brainflow_input_params::BrainFlowInputParamsBuilder, BoardIds,
    BrainFlowPresets,
};
use std::collections::HashMap;
use std::env;

use crate::domain::{models::eeg_work_modes::WorkMode, ports::input::eeg_headset::EegHeadsetPort};

// Default MAC address if environment variable is not set
const DEFAULT_DEVICE_MAC: &str = "C8:8F:B6:6D:E1:E2"; // Or another sensible default

#[injectable]
pub struct BrainFlowAdapter {
    board: BoardShim,
    work_mode: WorkMode,
}

#[implements]
impl BrainFlowAdapter {
    /// Factory function for creating the BrainFlowAdapter.
    /// Reads configuration from environment variables.
    #[factory]
    fn new() -> Self {
        // Logic moved from the old Default::default()
        let mac_address = env::var("BRAINBIT_MAC_ADDRESS").unwrap_or_else(|_| {
            println!(
                "BRAINBIT_MAC_ADDRESS not set, using default: {}",
                DEFAULT_DEVICE_MAC
            );
            DEFAULT_DEVICE_MAC.to_string()
        });

        println!("Using MAC Address: {}", mac_address);

        let params = BrainFlowInputParamsBuilder::default()
            .mac_address(mac_address)
            .timeout(20)
            .build();

        let board_id = BoardIds::BrainbitBoard;
        let board = BoardShim::new(board_id, params)
            .map_err(|e| format!("Failed to initialize BoardShim: {}", e))
            .expect("BoardShim initialization failed");
        board
            .prepare_session()
            .map_err(|e| format!("Failed to prepare session: {}", e))
            .expect("Session preparation failed");

        let instance = Self {
            board,
            work_mode: WorkMode::Calibration,
        };

        // Attempt to set initial mode
        // Consider if this initial command sending belongs in the factory or an init method
        let _ = instance._send_board_command("CommandStartResist");

        instance
    }

    /// Sends a configuration command to the board and handles the result.
    fn _send_board_command(&self, command: &str) -> Result<String, String> {
        println!("Sending command to board: {}", command);
        match self.board.config_board(command) {
            Ok(response) => {
                println!("Command '{}' successful. Response: {}", command, response);
                Ok(response)
            }
            Err(e) => {
                let error_msg = format!("Error sending command '{}': {}", command, e);
                eprintln!("{}", error_msg);
                Err(error_msg)
            }
        }
    }
}

#[implements]
impl EegHeadsetPort for BrainFlowAdapter {
    fn extract_impedance_data(&self) -> Result<HashMap<String, Vec<f32>>, String> {
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

        let data = self
            .board
            .get_board_data(Some(100), BrainFlowPresets::DefaultPreset)
            .map_err(|e| format!("Failed to get board data for impedance: {}", e))?;

        let mut impedance_values = HashMap::new();

        if data.shape()[0] == 0 {
            return Err("No data returned from board for impedance check.".to_string());
        }

        for (electrode_name, &channel_index) in electrode_channel_map.iter() {
            if channel_index < data.shape()[0] {
                let resistance_values_ohm: Vec<f64> =
                    data.row(channel_index).iter().map(|&v| v.abs()).collect();
                let resistance_values_kohm: Vec<f32> = resistance_values_ohm
                    .iter()
                    .map(|&v| (v / 1000.0) as f32)
                    .collect();
                impedance_values.insert(electrode_name.to_string(), resistance_values_kohm);
            } else {
                eprintln!(
                    "Warning: Resistance channel index {} for {} out of bounds (rows: {})",
                    channel_index,
                    electrode_name,
                    data.shape()[0]
                );
                impedance_values.insert(electrode_name.to_string(), vec![f32::NAN]);
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

        let data = self
            .board
            .get_board_data(None, BrainFlowPresets::DefaultPreset)
            .map_err(|e| format!("Failed to get board data for raw extraction: {}", e))?;

        let mut raw_data_map = HashMap::new();

        if data.shape()[0] == 0 {
            eprintln!("Warning: No new raw data returned from get_board_data.");
            return Ok(raw_data_map);
        }

        for (&channel_index, channel_name) in channel_map.iter() {
            if channel_index < data.shape()[0] {
                let channel_data_f64 = data.row(channel_index);
                let channel_data_f32: Vec<f32> =
                    channel_data_f64.iter().map(|&v| v as f32).collect();
                raw_data_map.insert(channel_name.clone(), channel_data_f32);
            } else {
                eprintln!(
                    "Error: EEG Channel index {} ('{}') out of bounds for data rows {}",
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
            println!("Already in {:?} mode.", new_mode);
            return; // Or return Ok(()) if the function returns Result
        }

        println!(
            "Attempting to change work mode from {:?} to {:?}",
            self.work_mode, new_mode
        );

        // 1. Send STOP command for the CURRENT mode
        let stop_command = match self.work_mode {
            WorkMode::Calibration => "CommandStopSignal",
            WorkMode::Extraction => "CommandStopResist",
        };

        // Use the private helper function. Abort if stop command fails.
        if self._send_board_command(stop_command).is_err() {
            eprintln!("Mode change aborted due to error stopping current mode.");
            return;
        }

        // 2. Send START command for the NEW mode
        let start_command = match new_mode {
            WorkMode::Calibration => "CommandStartResist",
            WorkMode::Extraction => "CommandStartSignal",
        };

        // Use the private helper function. Update state only on success.
        if self._send_board_command(start_command).is_ok() {
            println!("Successfully changed adapter state to {:?}", new_mode);
            self.work_mode = new_mode;
            // Optional delay after starting
            // std::thread::sleep(std::time::Duration::from_millis(500));
        } else {
            eprintln!(
                "Mode change failed. Adapter state remains {:?}.",
                self.work_mode
            );
            // State remains unchanged
        }
    }
}

// Ensure the board is stopped and released when the adapter is dropped
impl Drop for BrainFlowAdapter {
    fn drop(&mut self) {
        println!("Dropping BrainFlowAdapter, releasing session...");
        if self.board.is_prepared().unwrap_or(false) {
            let _ = self.board.stop_stream(); // Ignore error on stop
            if let Err(e) = self.board.release_session() {
                eprintln!("Error releasing BrainFlow session: {}", e);
            }
        }
    }
}
