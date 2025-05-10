use crate::domain::{
    commands::update_light_status_command::UpdateLightStatusCommand,
    context::NeuralAnalyticsContext, models::bulb_state::BulbState,
};
use log::info;
use presage::{command_handler, Error, Events};

/// This use case is responsible for updating the light status of a smart bulb.
/// It checks if the command is valid and then sends the appropriate command
/// to the smart bulb adapter to change its state.
/// 
/// # Arguments
/// * `_context`: A mutable reference to the `NeuralAnalyticsContext` which contains
/// the smart bulb adapter.
/// * `command`: The command to update the light status.
/// 
/// # Returns
/// * `Result<Events, Error>`: A result containing either the events generated from
/// the update or an error if something goes wrong.
#[command_handler(error = Error)]
pub async fn update_light_status_use_case(
    _context: &mut NeuralAnalyticsContext,
    command: UpdateLightStatusCommand,
) -> Result<Events, Error> {
    // Parse the command to extract the desired light status
    match command.is_light_on {
        true => {
            info!("Turning the light on...");

            // Obtain the smart bulb adapter from the context
            let smart_bulb = _context.smart_bulb_adapter.read().await;
            smart_bulb
                .change_state(BulbState::BulbOn)
                .await
                .map_err(|e| {
                    Error::MissingCommandHandler(Box::leak(e.to_string().into_boxed_str()))
                })?;
        }
        false => {
            info!("Turning the light off...");

            // Obtain the lock asynchronously for the change_state method
            let smart_bulb = _context.smart_bulb_adapter.read().await;
            smart_bulb
                .change_state(BulbState::BulbOff)
                .await
                .map_err(|e| {
                    Error::MissingCommandHandler(Box::leak(e.to_string().into_boxed_str()))
                })?;
        }
    }

    // Return an empty list of events for now
    Ok(Events::new())
}
