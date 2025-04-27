#[derive(Debug, PartialEq, Clone, Copy)] // Added Clone, Copy for convenience
pub enum WorkMode {
    Initialized,
    Calibration,
    Extraction,
}

impl WorkMode {
    pub fn to_string(&self) -> &str {
        match self {
            WorkMode::Calibration => "Calibration",
            WorkMode::Extraction => "Extraction",
            WorkMode::Initialized => "Initialized",
        }
    }

    pub fn from_string(mode: &str) -> Option<WorkMode> {
        match mode {
            "Calibration" => Some(WorkMode::Calibration),
            "Extraction" => Some(WorkMode::Extraction),
            "Initialized" => Some(WorkMode::Initialized),
            _ => None,
        }
    }
}
