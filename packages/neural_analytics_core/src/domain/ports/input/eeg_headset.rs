use std::collections::HashMap;

use crate::domain::models::eeg_work_modes::WorkMode;

pub trait EegHeadsetPort: Send + Sync + 'static {
    fn connect(&self) -> Result<(), String>;
    fn is_connected(&self) -> bool;
    fn disconnect(&mut self) -> Result<(), String>;
    fn extract_impedance_data(&self) -> Result<HashMap<String, u16>, String>;
    fn extract_raw_data(&self) -> Result<HashMap<String, Vec<f32>>, String>;
    fn change_work_mode(&mut self, mode: WorkMode);
    fn get_work_mode(&self) -> WorkMode;
}