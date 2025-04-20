use async_trait::async_trait;
use crate::domain::models::bulb_state::BulbState;

/// Defines the interface for controlling a smart bulb.
#[async_trait]
pub trait SmartBulbPort: Send + Sync + 'static {
    /// Changes the state of the smart bulb (on or off).
    ///
    /// # Arguments
    /// * `state` - The desired state (`BulbOn` or `BulbOff`).
    ///
    /// # Returns
    /// A Result indicating success (`Ok(())`) or failure (`Err(String)`).
    async fn change_state(&self, state: BulbState) -> Result<(), String>;
}
