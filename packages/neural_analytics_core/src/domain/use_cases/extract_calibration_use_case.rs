use presage::{command_handler, Error, Events};
use crate::domain::{
    commands::extract_calibration_data_command::ExtractCalibrationDataCommand, 
    context::NeuralAnalyticsContext, 
    models::{eeg_work_modes::WorkMode, event_internals::ReceivedCalibrationDataEvent}, 
    ports::input::eeg_headset::EegHeadsetPort
};
use std::collections::HashMap;
use log::{self, info};

#[command_handler(error = Error)]
pub async fn extract_calibration_data_use_case(
    _context: &mut NeuralAnalyticsContext,
    _command: ExtractCalibrationDataCommand,
) -> Result<Events, Error> {
    log::info!("Starting calibration data extraction from BrainBit device...");

    // Obtain the EEG headset adapter from the context
    let mut headset_guard = _context.eeg_headset_adapter.write().await;
    let headset: &mut dyn EegHeadsetPort = headset_guard.as_mut();

    // Check if the device is connected
    if !headset.is_connected() {
        let error_msg = "Error: Device is not connected. Connect first.";
        log::error!("{}", error_msg);
        return Err(Error::MissingCommandHandler(error_msg).into());
    }

    headset.change_work_mode(WorkMode::Calibration);
    
    let data = match headset.extract_impedance_data() {
        Ok(data) => {
            process_impedance_data(&data);
            log::info!("Calibration data successfully extracted.");
            data
        },
        Err(e) => {
            let error_msg = format!("Error extracting calibration data from device: {}", e);
            log::error!("{}", error_msg);
            return Err(Error::MissingCommandHandler(Box::leak(error_msg.into_boxed_str())).into());
        }
    };

    let mut events = Events::new();

    let _ = events.add(ReceivedCalibrationDataEvent {
        impedance_data: data,
    });

    Ok(events)
}

// Helper function to process impedance data
fn process_impedance_data(data: &HashMap<String, u16>) {
    info!("Processing electrode impedance data:");
    for (electrode, last_value) in data {            
        let status = if *last_value > 2 {
            "ERROR: Poor connection"
        } else if *last_value >= 1 && *last_value <= 2 {
            "WARNING: Acceptable connection"
        } else {
            "OK: Good connection"
        };
        
        info!("  Electrode {}: {:.2} kOhm - {}", electrode, last_value, status);
    }
}