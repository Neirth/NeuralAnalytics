#[derive(Debug)]
pub enum LightStatusError {
    InvalidStatus(String),
    CommunicationFailure(String),
}

impl std::fmt::Display for LightStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LightStatusError::InvalidStatus(status) => write!(f, "Invalid light status: {}", status),
            LightStatusError::CommunicationFailure(reason) => write!(f, "Failed to communicate with the light: {}", reason),
        }
    }
}

impl std::error::Error for LightStatusError {}
