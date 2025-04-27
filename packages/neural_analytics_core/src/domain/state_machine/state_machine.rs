use log::{debug, error, info};
use presage::{CommandBus, Configuration, Event};
use serde::de;
use statig::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    domain::{
        commands::{
            disconnect_headband_command::DisconnectHeadbandCommand, extract_calibration_data_command::ExtractCalibrationDataCommand, extract_generalist_data_command::ExtractGeneralistDataCommand, predict_color_thinking_command::PredictColorThinkingCommand, search_headband_command::SearchHeadbandCommand, update_light_status_command::UpdateLightStatusCommand
        },
        context::NeuralAnalyticsContext,
        events::{
            captured_headset_data_event::CapturedHeadsetDataEvent, headset_calibrated_event::HeadsetCalibratedEvent, headset_calibrating_event::HeadsetCalibratingEvent, headset_connected_event::HeadsetConnectedEvent, headset_disconnected_event::HeadsetDisconnectedEvent, initialized_core_event::InitializedCoreEvent
        },
        use_cases::{
            disconnect_headband_use_case::disconnect_headband_use_case, extract_calibration_use_case::extract_calibration_data_use_case, extract_extraction_use_case::extract_generalist_data_use_case, search_headband_use_case::search_headband_use_case, update_light_status_use_case::update_light_status_use_case
        },
    },
    EventData, INTERNAL_EVENT_HANDLER,
};

use super::neural_events::NeuralAnalyticsCoreEvents;

/// Main state machine - Initializes and holds DI container internally.
// Not injectable itself via blackbox_di attributes
pub(crate) struct MainStateMachine {
    context: Arc<Mutex<NeuralAnalyticsContext>>,
    command_bus: CommandBus<NeuralAnalyticsContext, presage::Error>,
}

#[state_machine(initial = "State::initialize_application()", state(derive(Debug)))]
impl MainStateMachine {
    /// Creates a new instance of the MainStateMachine asynchronously,
    /// building the necessary DI container.
    pub async fn new() -> Self {
        debug!("Initializate state machine...");

        let bus = CommandBus::<NeuralAnalyticsContext, presage::Error>::new().configure(
            Configuration::new()
                .command_handler(&disconnect_headband_use_case)
                .command_handler(&extract_calibration_data_use_case)
                .command_handler(&extract_generalist_data_use_case)
                .command_handler(&search_headband_use_case)
                .command_handler(&update_light_status_use_case),
        );

        Self {
            context: Arc::new(Mutex::new(NeuralAnalyticsContext::default())),
            command_bus: bus,
        }
    }

    /// Initialization state for the Neural Analytics system.
    /// This state initializes all necessary hardware components
    /// and prepares the system for operation.
    ///
    /// # State Flow
    /// - Executes `InitializeHardwarePartsCommand`
    /// - Emits `InitializedCoreEvent` upon successful initialization
    /// - Transitions to `awaiting_headset_connection` state
    #[state]
    #[allow(unused_variables)]
    async fn initialize_application(
        &mut self,
        event: &NeuralAnalyticsCoreEvents,
    ) -> Response<State> {
        // Initialization state - Detailed logging
        debug!("Executing state: initialize_application");

        if let Err(e) = send_event(
            &InitializedCoreEvent::NAME.to_string(),
            &EventData::default(),
        ) {
            error!("Failed to send initialized core event: {}", e);
            debug!("Repeating state: initialize_application due to error");
            return Transition(State::initialize_application());
        }

        debug!("Transitioning to state: awaiting_headset_connection");

        // Direct transition to the next state
        Transition(State::awaiting_headset_connection())
    }

    /// State that waits for a headset to connect to the system.
    /// This state continuously polls for available headsets
    /// and transitions to the calibration state when a connection is established.
    ///
    /// # State Flow
    /// - Executes `SearchHeadbandCommand` to find connected devices
    /// - Emits either `HeadsetConnectedEvent` or `HeadsetDisconnectedEvent`
    /// - On connection success, transitions to `awaiting_headset_calibration`
    /// - On connection failure, remains in `awaiting_headset_connection`
    #[state]
    #[allow(unused_variables)]
    async fn awaiting_headset_connection(
        &mut self,
        event: &NeuralAnalyticsCoreEvents,
    ) -> Response<State> {
        debug!("Executing state: awaiting_headset_connection");
        debug!("Disconnecting headset...");
        
        let disconnect_result = {
            let mut ctx = self.context.lock().await;
            self.command_bus
                .execute(&mut *ctx, DisconnectHeadbandCommand)
                .await
        };

        info!("Searching for headset...");

        let search_result = {
            let mut ctx = self.context.lock().await;
            self.command_bus
                .execute(&mut *ctx, SearchHeadbandCommand)
                .await
        };

        match search_result {
            Ok(_) => {
                // Headset connected
                info!("Headset correctly connected");
                if let Err(e) = send_event(
                    &HeadsetConnectedEvent::NAME.to_string(),
                    &EventData::default(),
                ) {
                    error!("Failed to send headset connected event: {}", e);

                    Transition(State::awaiting_headset_connection())
                } else {
                    debug!("Transitioning to state: awaiting_headset_calibration");
                    Transition(State::awaiting_headset_calibration())
                }
            }
            Err(_) => {
                // Headset disconnected
                info!("Headset not connected");

                if let Err(e) = send_event(
                    &HeadsetDisconnectedEvent::NAME.to_string(),
                    &EventData::default(),
                ) {
                    error!("Failed to send headset disconnected event: {}", e);
                }

                Transition(State::awaiting_headset_connection())
            }
        }
    }

    /// State that handles the headset calibration process.
    /// This state verifies that the headset's impedance levels are
    /// within acceptable ranges before allowing data capture.
    ///
    /// # State Flow
    /// - Executes `ExtractCalibrationDataCommand` to obtain impedance data
    /// - Analyzes impedance values to determine if calibration is acceptable
    /// - If calibration fails due to connection issues, returns to `awaiting_headset_connection`
    /// - If impedance values are too high (> 1000), emits `HeadsetCalibratingEvent` and remains in this state
    /// - If impedance values are acceptable, transitions to `capturing_headset_data`
    #[state]
    #[allow(unused_variables)]
    async fn awaiting_headset_calibration(
        &mut self,
        event: &NeuralAnalyticsCoreEvents,
    ) -> Response<State> {
        // Send debug message
        debug!("Executing state: awaiting_headset_calibration");

        // Get calibration data from internal context
        let calibration_result = {
            let mut ctx = self.context.lock().await;
            self.command_bus
                .execute(&mut *ctx, ExtractCalibrationDataCommand)
                .await
        };

        if calibration_result.is_err() {
            if let Err(e) = send_event(
                &HeadsetDisconnectedEvent::NAME.to_string(),
                &EventData::default(),
            ) {
                error!("Failed to send headset disconnected event: {}", e);
            }

            return Transition(State::awaiting_headset_connection());
        }

        // Get impedance data from internal context
        let impedance_data = {
            let ctx = self.context.lock().await;
            ctx.impedance_data.clone()
        };

        if let Some(data) = impedance_data {
            let needs_more_calibration = data.values().any(|&value| value > 1000 || value < 1);

            if needs_more_calibration {
                if let Err(e) = send_event(
                    &HeadsetCalibratingEvent::NAME.to_string(),
                    &EventData {
                        impedance_data: Some(data),
                        ..Default::default()
                    },
                ) {
                    error!("Failed to send headset calibrating event: {}", e);
                }

                return Transition(State::awaiting_headset_calibration());
            }
        }

        // If we get here, the device is calibrated
        if let Err(e) = send_event(
            &HeadsetCalibratedEvent::NAME.to_string(),
            &EventData::default(),
        ) {
            error!("Failed to send headset calibrated event: {}", e);
        }

        Transition(State::capturing_headset_data())
    }

    /// State for capturing and processing neural data from the headset.
    /// This state continuously retrieves EEG data, runs it through the
    /// machine learning model for color prediction, and controls output devices.
    ///
    /// # State Flow
    /// - Executes `ExtractGeneralistDataCommand` to get raw EEG data
    /// - If data extraction fails, returns to `awaiting_headset_connection`
    /// - Runs `PredictColorThinkingCommand` to process the data
    /// - Controls light status based on prediction ("green" = on)
    /// - Emits `CapturedHeadsetDataEvent` with processed data
    /// - Continues in this state in a loop to capture more data
    #[state]
    #[allow(unused_variables)]
    async fn capturing_headset_data(
        &mut self,
        event: &NeuralAnalyticsCoreEvents,
    ) -> Response<State> {
        let extract_result = {
            let mut ctx = self.context.lock().await;
            self.command_bus
                .execute(&mut *ctx, ExtractGeneralistDataCommand)
                .await
        };

        if extract_result.is_err() {
            if let Err(e) = send_event(
                &HeadsetDisconnectedEvent::NAME.to_string(),
                &EventData::default(),
            ) {
                error!("Failed to send headset disconnected event: {}", e);
            }

            return Transition(State::awaiting_headset_connection());
        }

        let raw_data = {
            let ctx = self.context.lock().await;
            ctx.headset_data.clone().unwrap_or_default()
        };

        // let color_prediction = {
        //     let mut ctx = self.context.lock().await;
        //     let prediction_result = self
        //         .command_bus
        //         .execute(&mut *ctx, PredictColorThinkingCommand {})
        //         .await;

        //     if let Err(e) = prediction_result {
        //         error!("Failed to predict color thinking: {:?}", e);
        //         return Transition(State::capturing_headset_data());
        //     }

        //     ctx.color_thinking.clone().unwrap_or_default()
        // };

        let color_prediction = "green".to_string(); // Placeholder for actual prediction logic

        if !color_prediction.is_empty() {
            let is_green = color_prediction == "green";
            let mut ctx = self.context.lock().await;

            if let Err(e) = self
                .command_bus
                .execute(
                    &mut *ctx,
                    UpdateLightStatusCommand {
                        is_light_on: is_green,
                    },
                )
                .await
            {
                error!("Failed to update light status: {:?}", e);
            }
        }

        if let Err(e) = send_event(
            &CapturedHeadsetDataEvent::NAME.to_string(),
            &EventData {
                headset_data: Some(raw_data),
                color_thinking: Some(color_prediction),
                impedance_data: None,
            },
        ) {
            error!("Failed to send captured headset data event: {}", e);
        }

        Transition(State::capturing_headset_data())
    }
}

/// Helper function to send events to external subscribers.
/// This delegates the event to the globally registered event handler.
///
/// # Parameters
/// - `event`: Event name/identifier
/// - `data`: Event payload data
///
/// # Returns
/// - `Result<(), String>`: Success or error message
fn send_event(event: &String, data: &EventData) -> Result<(), String> {
    // Send the event to the event handler
    if let Some(event_handler) = unsafe { INTERNAL_EVENT_HANDLER.as_ref() } {
        let result = event_handler(event, data);
        if let Err(ref e) = result {
            error!("Error sending event '{}': {}", event, e);
        } else {
            debug!("Event '{}' sent successfully", event);
        }
        result
    } else {
        Err("BUG: Event handler not set".to_string())
    }
}
