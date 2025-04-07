use std::collections::HashMap;

// port.rs
use crate::domain::models::eeg_work_modes::WorkMode;

pub trait EegHeadsetPort {
    fn extract_impedance_data(&self) -> Result<HashMap<String, Vec<f32>>, String>;
    fn extract_raw_data(&self) -> Result<HashMap<String, Vec<f32>>, String>;
    fn change_work_mode(&mut self, mode: WorkMode);
}