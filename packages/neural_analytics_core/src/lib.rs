use domain::events::{
    handle_captured_headset_data_event,
    handle_headset_calibrated_event,
    handle_headset_calibrating_event,
    handle_headset_connected_event,
    handle_headset_disconnected_event,
    handle_initialized_core_event
};
use domain::models::event_data::EventData;
use domain::state_machine::state_machine::MainStateMachine;
use domain::use_cases::{
    extract_calibration_use_case::ExtractCalibrationDataUseCase,
    extract_extraction_use_case::ExtractGeneralistDataUseCase,
    initialize_hardware_parts_use_case::InitializeHardwarePartsUseCase,
    predict_color_thinking_use_case::PredictColorThinkingUseCase,
    search_headband_use_case::SearchHeadbandUseCase,
    update_light_status_use_case::UpdateLightStatusUseCase
};
use presage::{CommandBus, Configuration};
use statig::prelude::{IntoStateMachineExt, StateMachine};

pub mod domain;
pub mod infrastructure;
pub mod utils;

// Internal State Machine
pub static mut INTERNAL_STATE_MACHINE: Option<StateMachine<MainStateMachine>> = None;

// Internal Command Bus
pub static INTERNAL_COMMAND_BUS: std::sync::LazyLock<CommandBus<CommandBus<presage::Error, presage::Error>, presage::Error>> = std::sync::LazyLock::new(|| {
    initialize_event_bus()
});


// Setted by the initialize_core function
pub static mut INTERNAL_EVENT_HANDLER: Option<fn(&str, &EventData) -> Result<(), String>> = None;
    
// Receives the event handler (function) and returns a command bus (function too)
pub(crate) fn initialize_core(
    event_handler: fn(&str, &EventData) -> Result<(), String>,
) -> Result<(), String> {
    // Initialize the state machine
    let state_machine = MainStateMachine::default().state_machine();

    // Set the event handler
    unsafe {
        INTERNAL_STATE_MACHINE = Some(state_machine);
        INTERNAL_EVENT_HANDLER = Some(event_handler);
    }
    
    // NOTE: No returns a external Command Bus because no intents are defined in GUI.
    Ok(())
}


fn initialize_event_bus() -> CommandBus<CommandBus<presage::Error, presage::Error>, presage::Error> {
    let configuration = Configuration::new()
        // Command handlers
        .command_handler(&ExtractCalibrationDataUseCase)       
        .command_handler(&ExtractGeneralistDataUseCase)
        .command_handler(&InitializeHardwarePartsUseCase)
        .command_handler(&PredictColorThinkingUseCase)
        .command_handler(&SearchHeadbandUseCase)
        .command_handler(&UpdateLightStatusUseCase)

        // Event handlers
        .event_handler(&handle_captured_headset_data_event)
        .event_handler(&handle_headset_calibrated_event)
        .event_handler(&handle_headset_calibrating_event)
        .event_handler(&handle_headset_connected_event)
        .event_handler(&handle_headset_disconnected_event)
        .event_handler(&handle_initialized_core_event);
    
    let command_bus = CommandBus::new().configure(configuration);

    command_bus
}