use log::{debug, error, info};
use presage::{command_handler, Error, Events};
use crate::domain::{commands::search_headband_command::SearchHeadbandCommand, context::NeuralAnalyticsContext};

/// This use case is responsible for searching and connecting to the EEG headset (BrainBit device).
/// It checks if the device is already connected and attempts to connect it.
/// If successful, it returns an empty list of events.
/// 
/// # Arguments
/// * `_context`: A mutable reference to the `NeuralAnalyticsContext` which contains
/// the EEG headset adapter.
/// * `_command`: The command to search and connect the headband.
/// 
/// # Returns
/// * `Result<Events, Error>`: A result containing either the events generated from
/// the connection or an error if something goes wrong.
#[command_handler(error = Error)]
pub async fn search_headband_use_case(
    _context: &mut NeuralAnalyticsContext,
    _command: SearchHeadbandCommand,
) -> Result<Events, Error> {
    info!("Starting search and connection of BrainBit device...");

    // Get the EEG headset adapter from the context
    let headset = _context.eeg_headset_adapter.read().await;

    // Check if already connected
    if headset.is_connected() {
        debug!("The device is already connected.");
        return Ok(Events::new());
    }

    // Try to connect to the device
    match headset.connect() {
        Ok(_) => {
            debug!("Connection established successfully.");
        },
        Err(e) => {
            let error_msg = format!("Error connecting to the device: {}", e);
            error!("{}", error_msg);
            return Err(Error::MissingCommandHandler(Box::leak(error_msg.into_boxed_str())).into());
        }
    }

    if headset.is_connected() {
        debug!("The device is now connected.");
        
        // Return an empty list of events for now
        Ok(Events::new())
    } else {
        let error_msg = "Error: Device is not connected or is not sending data. Connect first.";
        error!("{}", error_msg);
        return Err(Error::MissingCommandHandler(error_msg).into());
    }
}