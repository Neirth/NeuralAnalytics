use presage::{command_handler, Error, Events};
use crate::domain::{
    commands::predict_color_thinking_command::PredictColorThinkingCommand,
    context::NeuralAnalyticsContext,
    models::event_internals::ReceivedPredictColorThinkingDataEvent
};
use log::{info, error};

#[command_handler(error = Error)]
pub async fn predict_color_thinking_use_case(
    _context: &mut NeuralAnalyticsContext,
    _command: PredictColorThinkingCommand,
) -> Result<Events, Error> {
    info!("Starting color prediction for what the user is thinking...");

    // Check if EEG data is available
    let headset_data = match &_context.headset_data {
        Some(data) => data,
        None => {
            let error_msg = "No EEG data available for prediction";
            error!("{}", error_msg);
            return Err(Error::MissingCommandHandler(error_msg).into());
        }
    };

    // Use the inference service to predict the color
    info!("Processing EEG data for prediction...");
    let color_result = _context.model_service.predict_color(headset_data)
        .map_err(|e| {
            let error_msg = format!("Error predicting color: {}", e);
            error!("{}", error_msg);
            Error::MissingCommandHandler(Box::leak(error_msg.into_boxed_str()))
        })?;

    // Save the result in the context
    _context.color_thinking = Some(color_result.clone());
    info!("Successful prediction: the user is thinking of the color '{}'", color_result);

    // Create and return events
    let mut events = Events::new();
    let _ = events.add( ReceivedPredictColorThinkingDataEvent {
        color_thinking: color_result,
    });
    
    // Send the event to the event queue
    Ok(events)
}