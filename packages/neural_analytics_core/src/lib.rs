use domain::context::NeuralAnalyticsContext;
use domain::models::event_data::EventData;
use domain::state_machine::{
    state_machine::MainStateMachine,
    neural_events::NeuralAnalyticsCoreEvents
};
use domain::use_cases::{
    extract_calibration_use_case::ExtractCalibrationDataUseCase,
    extract_extraction_use_case::ExtractGeneralistDataUseCase,
    initialize_hardware_parts_use_case::InitializeHardwarePartsUseCase,
    predict_color_thinking_use_case::PredictColorThinkingUseCase,
    search_headband_use_case::SearchHeadbandUseCase,
    update_light_status_use_case::UpdateLightStatusUseCase
};
use presage::{CommandBus, Configuration};
use statig::awaitable::{IntoStateMachineExt, StateMachine};

pub mod domain;
pub mod infrastructure;
pub mod utils;

// Internal State Machine
pub(crate) static mut INTERNAL_STATE_MACHINE: Option<StateMachine<MainStateMachine>> = None;

// Internal Command Bus
pub(crate) static INTERNAL_COMMAND_BUS: std::sync::LazyLock<CommandBus<NeuralAnalyticsContext, presage::Error>> = std::sync::LazyLock::new(|| {
    CommandBus::<NeuralAnalyticsContext, presage::Error>::new().configure(
        Configuration::<NeuralAnalyticsContext, presage::Error>::new()
            // Command handlers
            .command_handler(&ExtractCalibrationDataUseCase)       
            .command_handler(&ExtractGeneralistDataUseCase)
            .command_handler(&InitializeHardwarePartsUseCase)
            .command_handler(&PredictColorThinkingUseCase)
            .command_handler(&SearchHeadbandUseCase)
            .command_handler(&UpdateLightStatusUseCase)
    )}
);


// Setted by the initialize_core function
pub(crate) static mut INTERNAL_EVENT_HANDLER: Option<Box<dyn Fn(&String, &EventData) -> Result<(), String> + Send>> = None;
    
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
    // Define the state machine
    let mut raw_state_machine = MainStateMachine::default().state_machine();


    // Set the event handler
    unsafe {
        // Set the event handler to the static variable
        INTERNAL_STATE_MACHINE = Some(raw_state_machine);
        INTERNAL_EVENT_HANDLER = Some(Box::new(event_handler));
    }

    unsafe {
        // Initialize the state machine
        INTERNAL_STATE_MACHINE.as_mut().unwrap().handle(&NeuralAnalyticsCoreEvents::InitializeCore).await;
    }

    // NOTE: No returns a external Command Bus because no intents are defined in GUI.
    Ok(())
}
