use std::collections::HashMap;
use log::{info, warn};
use rand::{Rng, thread_rng};
use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::domain::{models::eeg_work_modes::WorkMode, ports::input::eeg_headset::EegHeadsetPort};

// Mutex to maintain consistency of simulated data between calls
static SIMULATED_DATA: Lazy<Mutex<SimulatedEegData>> = Lazy::new(|| {
    Mutex::new(SimulatedEegData::new())
});

// Structure that stores simulated EEG data
struct SimulatedEegData {
    raw_data_buffer: HashMap<String, Vec<f32>>,
    impedance_values: HashMap<String, u16>,
}

impl SimulatedEegData {
    fn new() -> Self {
        let channels = vec!["T3", "T4", "O1", "O2"];
        let mut raw_data_buffer = HashMap::new();
        let mut impedance_values = HashMap::new();
        
        let mut rng = thread_rng();
        
        // Initialize simulated data for each channel
        for &channel in &channels {
            // Generate simulated EEG data (500 samples per channel)
            let mut channel_data = Vec::with_capacity(500);
            for _ in 0..500 {
                // Typical EEG values are in microvolts (generally between -100 and 100 µV)
                channel_data.push(rng.gen_range(-100.0..100.0));
            }
            raw_data_buffer.insert(channel.to_string(), channel_data);
            
            // Generate simulated impedance values (in kOhm)
            // Typical values for good connection are below 10 kOhm
            impedance_values.insert(channel.to_string(), rng.gen_range(1..15));
        }
        
        Self {
            raw_data_buffer,
            impedance_values,
        }
    }
    
    // Generate new random data to simulate changes in signals
    fn refresh_data(&mut self) {
        let mut rng = thread_rng();
        
        // Update EEG data
        for (_channel, data) in self.raw_data_buffer.iter_mut() {
            // Simulate a signal that varies slightly between samples
            let base = data.last().copied().unwrap_or(0.0);
            let next_value = base + rng.gen_range(-5.0..5.0);
            // Keep values within a reasonable range
            let bounded_value = next_value.max(-100.0).min(100.0);
            
            // Remove the oldest sample and add the new one
            if data.len() >= 500 {
                data.remove(0);
            }
            data.push(bounded_value);
        }
        
        // Occasionally update impedance values
        if rng.gen_bool(0.1) { // 10% chance of change
            for (_channel, impedance) in self.impedance_values.iter_mut() {
                // Simulate small changes in impedance
                let change = rng.gen_range(-2..3);
                *impedance = (*impedance as i16 + change).max(1).min(20) as u16;
            }
        }
    }
}

/// Mock adapter to simulate the operation of an EEG device
/// when real hardware is not available
pub struct MockHeadsetAdapter {
    work_mode: WorkMode,
    is_connected: bool,
}

impl Default for MockHeadsetAdapter {
    fn default() -> Self {
        info!("Creating mock adapter for EEG");
        Self {
            work_mode: WorkMode::Calibration,
            is_connected: true, // By default, we simulate it's already connected
        }
    }
}

impl EegHeadsetPort for MockHeadsetAdapter {
    fn extract_impedance_data(&self) -> Result<HashMap<String, u16>, String> {
        if !matches!(self.work_mode, WorkMode::Calibration) {
            return Err("Device not in Calibration mode. Call change_work_mode first.".to_string());
        }
        
        if !self.is_connected {
            return Err("Device is not connected".to_string());
        }
        
        // Get and update simulated data
        let mut simulated_data = SIMULATED_DATA.lock().unwrap();
        simulated_data.refresh_data();
        
        // Clone impedance values to return them
        let impedance_values = simulated_data.impedance_values.clone();
        
        info!("Mock: Extracting simulated impedance data: {:?}", impedance_values);
        Ok(impedance_values)
    }

    fn extract_raw_data(&self) -> Result<HashMap<String, Vec<f32>>, String> {
        if !matches!(self.work_mode, WorkMode::Extraction) {
            return Err("Device not in Extraction mode. Call change_work_mode first.".to_string());
        }
        
        if !self.is_connected {
            return Err("Device is not connected".to_string());
        }
        
        // Get and update simulated data
        let mut simulated_data = SIMULATED_DATA.lock().unwrap();
        simulated_data.refresh_data();
        
        // Clone EEG data to return them
        let raw_data = simulated_data.raw_data_buffer.clone();
        
        info!("Mock: Extracting simulated EEG data from {} channels", raw_data.len());
        Ok(raw_data)
    }

    fn change_work_mode(&mut self, new_mode: WorkMode) {
        if self.work_mode == new_mode {
            info!("Mock: Already in {:?} mode", new_mode);
            return;
        }
        
        info!("Mock: Changing from {:?} mode to {:?}", self.work_mode, new_mode);
        self.work_mode = new_mode;
    }

    fn connect(&self) -> Result<(), String> {
        if self.is_connected {
            warn!("Mock: Device is already connected");
            return Ok(());
        }
        
        // Here we could simulate a random connection error
        info!("Mock: Connecting to simulated device");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        info!("Mock: Checking connection - {}", self.is_connected);
        self.is_connected
    }

    fn disconnect(&mut self) -> Result<(), String> {
        if !self.is_connected {
            warn!("Mock: The device is not connected");
            return Err("Device is not connected".to_string());
        }
        
        info!("Mock: Disconnecting simulated device");
        Ok(())
    }
    
    fn get_work_mode(&self) -> WorkMode {
        self.work_mode
    }
}

impl Drop for MockHeadsetAdapter {
    fn drop(&mut self) {
        info!("Mock: Releasing mock adapter resources");
    }
}
