use std::sync::Arc;

use presage::Event;
use statig::prelude::*;
use tokio::sync::Mutex;
use log::error;

use crate::{
    domain::{
        commands::{
            extract_calibration_data_command::ExtractCalibrationDataCommand, 
            extract_generalist_data_command::ExtractGeneralistDataCommand, 
            initialize_hardware_parts_command::InitializeHardwarePartsCommand, 
            predict_color_thinking_command::PredictColorThinkingCommand, 
            search_headband_command::SearchHeadbandCommand, 
            update_light_status_command::UpdateLightStatusCommand
        }, context::NeuralAnalyticsContext, events::{
            captured_headset_data_event::CapturedHeadsetDataEvent, 
            headset_calibrating_event::HeadsetCalibratingEvent, 
            headset_connected_event::HeadsetConnectedEvent, 
            headset_disconnected_event::HeadsetDisconnectedEvent, 
            initialized_core_event::InitializedCoreEvent
        }
    }, EventData, INTERNAL_COMMAND_BUS, INTERNAL_EVENT_HANDLER
};

use super::neural_events::NeuralAnalyticsCoreEvents;

/// Main state machine that controls the Neural Analytics system workflow.
/// This state machine handles the progression between different operational 
/// states of the system, from initialization to data capture.
pub(crate) struct MainStateMachine {
    /// Shared context containing the application state and data
    context: Arc<Mutex<NeuralAnalyticsContext>>,
}

impl Default for MainStateMachine {
    fn default() -> Self {
        MainStateMachine {
            context: Arc::new(Mutex::new(NeuralAnalyticsContext::default())),
        }
    }
}

#[state_machine(initial = "State::initialize_application()", state(derive(Debug)))]
impl MainStateMachine {
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
    async fn initialize_application(&mut self, event: &NeuralAnalyticsCoreEvents) -> Response<State> {
        // Execute the command and release the context immediately
        let result = {
            let mut ctx = self.context.lock().await;
            INTERNAL_COMMAND_BUS.execute(&mut *ctx, InitializeHardwarePartsCommand).await
        };
        
        if let Err(e) = result {
            error!("Failed to initialize hardware parts: {:?}", e);
            return Transition(State::initialize_application());
        }

        // Send the event without keeping the context locked
        if let Err(e) = send_event(&InitializedCoreEvent::NAME.to_string(), &EventData {
            headset_data: None,
            color_thinking: None,
            impedance_data: None,
        }) {
            error!("Failed to send initialized core event: {}", e);
            return Transition(State::initialize_application());
        }

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
    async fn awaiting_headset_connection(&mut self, event: &NeuralAnalyticsCoreEvents) -> Response<State> {
        // Execute command to search for the headband
        let search_result = {
            let mut ctx = self.context.lock().await;
            INTERNAL_COMMAND_BUS.execute(&mut *ctx, SearchHeadbandCommand).await
        };
        
        match search_result {
            Ok(_) => {
                // Headset connected
                if let Err(e) = send_event(&HeadsetConnectedEvent::NAME.to_string(), &EventData {
                    headset_data: None,
                    color_thinking: None,
                    impedance_data: None,
                }) {
                    error!("Failed to send headset connected event: {}", e);
                    return Transition(State::awaiting_headset_connection());
                }
                
                Transition(State::awaiting_headset_calibration())
            },
            Err(_) => {
                // Headset disconnected
                if let Err(e) = send_event(&HeadsetDisconnectedEvent::NAME.to_string(), &EventData {
                    headset_data: None,
                    color_thinking: None,
                    impedance_data: None,
                }) {
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
    async fn awaiting_headset_calibration(&mut self, event: &NeuralAnalyticsCoreEvents) -> Response<State> {
        // Extraction of calibration data
        let calibration_result = {
            let mut ctx = self.context.lock().await;
            INTERNAL_COMMAND_BUS.execute(&mut *ctx, ExtractCalibrationDataCommand).await
        };
        
        // Check if extraction failed
        if calibration_result.is_err() {
            if let Err(e) = send_event(&HeadsetDisconnectedEvent::NAME.to_string(), &EventData {
                headset_data: None,
                color_thinking: None,
                impedance_data: None,
            }) {
                error!("Failed to send headset disconnected event: {}", e);
            }
            
            return Transition(State::awaiting_headset_connection());
        }
        
        // Get and analyze the impedance data
        let impedance_data = {
            let ctx = self.context.lock().await;
            ctx.impedance_data.clone()
        };
        
        if let Some(data) = impedance_data {
            // Check if any impedance value is greater than 1000
            if data.values().any(|value| *value > 1000) {
                if let Err(e) = send_event(&HeadsetCalibratingEvent::NAME.to_string(), &EventData {
                    headset_data: None,
                    color_thinking: None,
                    impedance_data: Some(data),
                }) {
                    error!("Failed to send headset calibrating event: {}", e);
                }
                
                return Transition(State::awaiting_headset_calibration());
            }
        }
        
        // If we get here, the device is calibrated
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
    async fn capturing_headset_data(&mut self, event: &NeuralAnalyticsCoreEvents) -> Response<State> {
        // Extract general data
        let extract_result = {
            let mut ctx = self.context.lock().await;
            INTERNAL_COMMAND_BUS.execute(&mut *ctx, ExtractGeneralistDataCommand).await
        };
        
        // Check if extraction failed
        if extract_result.is_err() {
            if let Err(e) = send_event(&HeadsetDisconnectedEvent::NAME.to_string(), &EventData {
                headset_data: None,
                color_thinking: None,
                impedance_data: None,
            }) {
                error!("Failed to send headset disconnected event: {}", e);
            }
            
            return Transition(State::awaiting_headset_connection());
        }
        
        // Get data from context
        let raw_data = {
            let ctx = self.context.lock().await;
            ctx.headset_data.clone().unwrap_or_default()
        };
        
        // Predict the thought color
        let color_prediction = {
            let mut ctx = self.context.lock().await;
            
            // Execute prediction command
            if let Err(e) = INTERNAL_COMMAND_BUS.execute(&mut *ctx, PredictColorThinkingCommand {
                headset_data: raw_data.clone(),
            }).await {
                error!("Failed to predict color thinking: {:?}", e);
                return Transition(State::capturing_headset_data());
            }
            
            ctx.color_thinking.clone().unwrap_or_default()
        };
        
        // Update light status if necessary
        if !color_prediction.is_empty() {
            let is_green = color_prediction == "green";
            let mut ctx = self.context.lock().await;
            
            if let Err(e) = INTERNAL_COMMAND_BUS.execute(&mut *ctx, UpdateLightStatusCommand {
                is_light_on: is_green,
            }).await {
                error!("Failed to update light status: {:?}", e);
            }
        }
        
        // Send event with captured data
        if let Err(e) = send_event(&CapturedHeadsetDataEvent::NAME.to_string(), &EventData {
            headset_data: Some(raw_data),
            color_thinking: Some(color_prediction),
            impedance_data: None,
        }) {
            error!("Failed to send captured headset data event: {}", e);
        }
        
        // Continue capturing
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
        event_handler(event, data)
    } else {
        Err("BUG: Event handler not set".to_string())
    }
}