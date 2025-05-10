
use domain::models::event_data::EventData;
use domain::state_machine::{
    neural_events::NeuralAnalyticsCoreEvents, state_machine::MainStateMachine,
};

use statig::awaitable::{InitializedStateMachine, IntoStateMachineExt};

pub mod domain;
pub mod infrastructure;
pub mod utils;

// Internal State Machine
pub(crate) static mut INTERNAL_STATE_MACHINE: Option<InitializedStateMachine<MainStateMachine>> = None;

// Setted by the initialize_core function
pub(crate) static mut INTERNAL_EVENT_HANDLER: Option<
    Box<dyn Fn(&String, &EventData) -> Result<(), String> + Send>,
> = None;

/// Initialize the core of the application
///
/// This function initializes the core of the application by setting up the state machine and the event handler.
/// It is called at the beginning of the application to set up the necessary components.
///
/// # Arguments
/// - `event_handler`: A function that handles events. It takes a string and an `EventData` struct as arguments and returns a `Result<(), String>`.
///
/// # Returns
/// - `Result<(), String>`: Returns `Ok(())` if the initialization is successful, or an error message if it fails.
///
pub async fn initialize_core<F>(event_handler: F) -> Result<(), String>
where
    F: Fn(&String, &EventData) -> Result<(), String> + 'static + Send,
{
    // Define the state machine asynchronously
    let state_machine_instance = MainStateMachine::new().await;
    let raw_state_machine = state_machine_instance.uninitialized_state_machine().init().await;

    unsafe {
        // Set the event handler to the static variable
        INTERNAL_STATE_MACHINE = Some(raw_state_machine);
        INTERNAL_EVENT_HANDLER = Some(Box::new(event_handler));

        // Initialize the state machine
        INTERNAL_STATE_MACHINE
            .as_mut()
            .unwrap()
            .handle(&NeuralAnalyticsCoreEvents::InitializeCore)
            .await;
    }

    tokio::spawn(async move {
        // Run the state machine in the background
        loop {
            unsafe {
                let state_machine = INTERNAL_STATE_MACHINE.as_mut().unwrap();
                state_machine.handle(&NeuralAnalyticsCoreEvents::BackgroundTick).await;
            }
        }
    });

    // NOTE: No returns a external Command Bus because no intents are defined in GUI.
    Ok(())
}
