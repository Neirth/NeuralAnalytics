use log::{debug, error, info};
use presage::{command_handler, Error, Events};
use crate::domain::{commands::disconnect_headband_command::DisconnectHeadbandCommand, context::NeuralAnalyticsContext};

#[command_handler(error = Error)]
pub async fn disconnect_headband_use_case(
    _context: &mut NeuralAnalyticsContext,
    _command: DisconnectHeadbandCommand,
) -> Result<Events, Error> {
    info!("Starting search and connection of BrainBit device...");

    // Obtain the EEG headset adapter from the context
    let mut headset = _context.eeg_headset_adapter.write().await;

    let is_connected = headset.is_connected();

    // Check if already connected
    if !is_connected {
        debug!("The device is already disconnected.");
        return Ok(Events::new());
    }

    // Try to connect to the device
    match headset.disconnect() {
        Ok(_) => {
            debug!("Disconnected successfully.");
        },
        Err(e) => {
            let error_msg = format!("Error disconnecting from the device: {}", e);
            error!("{}", error_msg);
            return Err(Error::MissingCommandHandler(Box::leak(error_msg.into_boxed_str())).into());
        }
    }

    if !headset.is_connected() {
        debug!("The device is now disconnected.");
        
        // Return an empty list of events for now
        Ok(Events::new())
    } else {
        let error_msg = "Error: Device is not disconnected or is sending data. Disconnect first.";
        error!("{}", error_msg);
        return Err(Error::MissingCommandHandler(error_msg).into());
    }
}