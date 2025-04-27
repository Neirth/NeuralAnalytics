use log::{info, error};
use presage::{command_handler, Error, Events};
use crate::domain::{commands::extract_generalist_data_command::ExtractGeneralistDataCommand, context::NeuralAnalyticsContext, models::{eeg_work_modes::WorkMode, event_internals::ReceivedGeneralistDataEvent}, ports::input::eeg_headset::EegHeadsetPort};
use std::collections::HashMap;

#[command_handler(error = Error)]
pub async fn extract_generalist_data_use_case(
    _context: &mut NeuralAnalyticsContext,
    _command: ExtractGeneralistDataCommand,
) -> Result<Events, Error> {
    info!("Starting raw data extraction from BrainBit device...");

    // Get the EEG headset adapter from the context
    let mut headset_guard = _context.eeg_headset_adapter.write().await;
    let headset: &mut dyn EegHeadsetPort = headset_guard.as_mut();
    
    // Check if the device is connected
    if !headset.is_connected() {
        let error_msg = "Error: Device is not connected. Connect first.";
        error!("{}", error_msg);
        return Err(Error::MissingCommandHandler(error_msg).into());
    }

    // Change to extraction mode before trying to get data
    headset.change_work_mode(WorkMode::Extraction);
    
    // Try to extract raw data from the device
    let data = match headset.extract_raw_data() {
        Ok(data) => {
            // Process the extracted data
            process_eeg_data(&data);
            data
        },
        Err(e) => {
            let error_msg = format!("Error extracting data from device: {}", e);
            error!("{}", error_msg);
            return Err(Error::MissingCommandHandler(Box::leak(error_msg.into_boxed_str())).into());
        }
    };

    // Create event with the extracted data
    let mut events = Events::new();
    let _ = events.add(ReceivedGeneralistDataEvent {
        headset_data: data,
    });

    // Send the event to the event queue
    Ok(events)
}

// Helper function to process the EEG data
fn process_eeg_data(data: &HashMap<String, Vec<f32>>) {
    // For now, we simply show basic information about the received data
    info!("Processing EEG data:");
    for (channel, values) in data {
        info!("  Channel {}: {} samples received", channel, values.len());
        if !values.is_empty() {
            info!("    - First values: {:?}", &values[..std::cmp::min(values.len(), 5)]);
        }
    }
}