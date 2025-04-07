use async_trait::async_trait;
use tapo::ApiClient;
use tapo::LightHandler;

use crate::domain::models::bulb_state::BulbState;
use crate::domain::ports::output::smart_bulb::SmartBulbPort;

/// Adapter for interacting with a Tapo smart bulb.
pub struct TapoSmartBulbAdapter {
    // Stores the specific handler for the Tapo light device.
    // LightHandler provides methods like on(), off(), set_brightness(), etc.
    device_client: LightHandler,
    // Store the IP address for potential retries or logging
    ip_address: String,
}

impl TapoSmartBulbAdapter {
    /// Creates a new adapter instance and connects to the Tapo device.
    ///
    /// # Arguments
    /// * `ip_address` - The IP address of the Tapo smart bulb.
    /// * `username` - Your Tapo cloud username (email).
    /// * `password` - Your Tapo cloud password.
    ///
    /// # Returns
    /// A Result containing the adapter instance or an error string.
    pub async fn new(
        ip_address: String,
        username: String,
        password: String,
    ) -> Result<Self, String> {
        println!(
            "Attempting to connect to Tapo device at {} with username {}",
            ip_address, username
        );

        // Create the Tapo API client
        let api_client = ApiClient::new(username, password);

        // Get a specific light handler for the device at the given IP.
        // Assuming L510 for now, adjust if using a different bulb model (e.g., l530, l610).
        let device_client = api_client
            .l510(ip_address.clone())
            .await
            // TODO: Replace unwrap() with proper error handling (e.g., map_err)
            .map_err(|e| format!("Failed to get Tapo light handler: {}", e))?;
        // .unwrap();

        println!("Successfully connected to Tapo device at {}", ip_address);

        Ok(Self {
            device_client,
            ip_address,
        })
    }
}

#[async_trait]
impl SmartBulbPort for TapoSmartBulbAdapter {
    /// Changes the state of the smart bulb (on or off).
    async fn change_state(&self, state: BulbState) -> Result<(), String> {
        println!(
            "Adapter: Changing state of bulb {} to {:?}",
            self.ip_address, state
        );

        let result = match state {
            BulbState::BulbOn => self.device_client.on().await,
            BulbState::BulbOff => self.device_client.off().await,
        };

        // Map the result from the tapo library to our Result<(), String>
        result.map_err(|e| {
            let error_msg = format!(
                "Failed to change Tapo bulb state to {:?} for device {}: {}",
                state, self.ip_address, e
            );
            eprintln!("{}", error_msg);
            error_msg
        })
    }
}
