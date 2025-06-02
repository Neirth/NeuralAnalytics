#[derive(Debug, PartialEq, Clone, Copy)] // Added Clone, Copy for convenience
pub enum WorkMode {
    Initialized,
    Calibration,
    Extraction,
}
