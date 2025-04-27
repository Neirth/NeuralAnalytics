use crate::domain::{
    commands::update_light_status_command::UpdateLightStatusCommand,
    context::NeuralAnalyticsContext, models::bulb_state::BulbState,
};
use log::info;
use presage::{command_handler, Error, Events};

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
        _ => {
            return Err(Error::MissingCommandHandler("Invalid light status").into());
        }
    }

    // Return an empty list of events for now
    Ok(Events::new())
}
