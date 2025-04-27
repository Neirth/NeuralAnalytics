use async_trait::async_trait;
use log::{debug, error};
use std::env;
use std::sync::Arc;
use tapo::{ApiClient, LightHandler};
use tokio::sync::Mutex;

use crate::domain::models::bulb_state::BulbState;
use crate::domain::ports::output::smart_bulb::SmartBulbPort;

/// Adapter for interacting with a Tapo smart bulb using environment variables.
/// Connection is initiated in the background when `new` is called.
pub struct TapoSmartBulbAdapter {
    // Stores the handler after background connection. Needs Arc<Mutex> for sharing.
    device_client: Arc<Mutex<Option<LightHandler>>>,
    // Keep config details for potential retries or reference
    ip_address: String,
    username: String,
    password: String,
}

impl Default for TapoSmartBulbAdapter {
    /// Creates a new instance and initiates connection in the background.
    /// Returns immediately. The adapter might not be connected yet.
    /// Panics if environment variables are not set.
    fn default() -> Self {
        debug!("Creating TapoSmartBulbAdapter config and spawning connection task...");

        let ip_address =
            env::var("TAPO_IP_ADDRESS").expect("TAPO_IP_ADDRESS environment variable not set");
        let username =
            env::var("TAPO_USERNAME").expect("TAPO_USERNAME environment variable not set");
        let password =
            env::var("TAPO_PASSWORD").expect("TAPO_PASSWORD environment variable not set");

        let device_client_arc = Arc::new(Mutex::new(None));

        // Clone data needed for the background task
        let ip_clone = ip_address.clone();
        let user_clone = username.clone();
        let pass_clone = password.clone();
        let client_arc_clone = Arc::clone(&device_client_arc);

        // Spawn the connection logic in a background task
        tokio::spawn(async move {
            debug!(
                "Background task: Attempting connection to Tapo device at {}",
                ip_clone
            );

            let api_client = ApiClient::new(user_clone, pass_clone);
            
            match api_client.l510(ip_clone.clone()).await {
                Ok(handler) => {
                    debug!(
                        "Background task: Successfully connected to Tapo device at {}. Updating adapter state.",
                        ip_clone
                    );
                    
                    // Lock the tokio mutex asynchronously
                    let mut client_guard = client_arc_clone.lock().await;
                    *client_guard = Some(handler);
                }
                Err(e) => {
                    // Log the error; the Option remains None
                    error!(
                        "Background task: Failed to connect to Tapo device {}: {}",
                        ip_clone, e
                    );
                }
            }
        });

        debug!(
            "TapoSmartBulbAdapter::new returning for IP: {}. Connection proceeds in background.",
            ip_address
        );

        Self {
            device_client: device_client_arc,
            ip_address,
            username,
            password,
        }
    }
}


#[async_trait]
impl SmartBulbPort for TapoSmartBulbAdapter {
    /// Changes the state of the smart bulb (on or off).
    /// Returns an error if the background connection hasn't completed successfully yet.
    async fn change_state(&self, state: BulbState) -> Result<(), String> {
        debug!(
            "Adapter: Requesting state change for bulb {} to {:?}",
            self.ip_address, state
        );

        // Lock the tokio mutex asynchronously
        let maybe_client_guard = self.device_client.lock().await;

        // Check if the client is available (connection successful)
        let client = maybe_client_guard.as_ref().ok_or_else(|| {
            format!(
                "Cannot change state for Tapo device {}: Not connected yet or connection failed.",
                self.ip_address
            )
        })?;

        // Proceed with the command using the handler from the Option
        let result = match state {
            BulbState::BulbOn => client.on().await,
            BulbState::BulbOff => client.off().await,
        };

        result.map_err(|e| {
            let error_msg = format!(
                "Failed to change Tapo bulb state to {:?} for device {}: {}",
                state, self.ip_address, e
            );
            error!("{}", error_msg);
            error_msg
        })
    }
}
