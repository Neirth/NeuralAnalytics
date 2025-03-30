use std::collections::HashMap;

use statig::prelude::*;

use crate::{
    domain::{
        commands::{
            extract_calibration_data_command::ExtractCalibrationDataCommand, 
            extract_generalist_data_command::ExtractGeneralistDataCommand, 
            initialize_hardware_parts_command::InitializeHardwarePartsCommand, 
            predict_color_thinking_command::PredictColorThinkingCommand, 
            search_headband_command::SearchHeadbandCommand, 
            update_light_status_command::UpdateLightStatusCommand
        }, events::{
            captured_headset_data_event::CapturedHeadsetDataEvent, 
            headset_calibrating_event::HeadsetCalibratingEvent, 
            headset_connected_event::HeadsetConnectedEvent, 
            headset_disconnected_event::HeadsetDisconnectedEvent, 
            initialized_core_event::InitializedCoreEvent
        }
    },
    INTERNAL_COMMAND_BUS
};

use super::neural_events::NeuralAnalyticsCoreEvents;

#[derive(Default)]
pub struct MainStateMachine;

/// State machine for the main state machine of the neural analytics core.
/// 
/// This state machine is responsible for managing the state of the neural analytics core.
/// 
/// The state machine has the following states:
/// - `initialize_application`: The initial state of the state machine.
/// - `awaiting_headset_connection`: The state where the state machine is waiting for the headset to be connected.
/// - `awaiting_headset_calibration`: The state where the state machine is waiting for the headset to be calibrated.
/// - `capturing_headset_data`: The state where the state machine is capturing data from the headset.
/// 
#[state_machine(initial = "State::initialize_application()")]
impl MainStateMachine {
    #[state(entry_action = "initialize_application_action")]
    fn initialize_application(event: &NeuralAnalyticsCoreEvents) -> Response<State> {
        match event {
            NeuralAnalyticsCoreEvents::InitializedCore => Transition(State::awaiting_headset_connection()),
            _ => Super,
        }
    }

    #[state(entry_action = "headset_connection_action")]
    fn awaiting_headset_connection(event: &NeuralAnalyticsCoreEvents) -> Response<State> {
        match event {
            NeuralAnalyticsCoreEvents::HeadsetConnected => Transition(State::awaiting_headset_calibration()),
            NeuralAnalyticsCoreEvents::HeadsetDisconnected => Transition(State::awaiting_headset_connection()),
            _ => Super,
        }
    }

    #[state(entry_action = "headset_calibration_action")]
    fn awaiting_headset_calibration(event: &NeuralAnalyticsCoreEvents) -> Response<State> {
        match event {
            NeuralAnalyticsCoreEvents::HeadsetDisconnected => Transition(State::awaiting_headset_connection()),
            NeuralAnalyticsCoreEvents::HeadsetCalibrated => Transition(State::capturing_headset_data()),
            NeuralAnalyticsCoreEvents::HeadsetCalibrating => Transition(State::awaiting_headset_calibration()),
            _ => Super,
        }
    }

    #[state(entry_action = "capture_data_action")]
    fn capturing_headset_data(event: &NeuralAnalyticsCoreEvents) -> Response<State> {
        match event {
            NeuralAnalyticsCoreEvents::HeadsetDisconnected => Transition(State::awaiting_headset_connection()),
            NeuralAnalyticsCoreEvents::CapturedHeadsetData => Transition(State::capturing_headset_data()),
            _ => Super,
        }
    }

    #[action]
    pub fn initialize_application_action(&self) {
        // Initialize the hardware parts
        INTERNAL_COMMAND_BUS
            .execute(InitializeHardwarePartsCommand)
            .expect("Failed to initialize hardware parts");

        // Send the initialized core event to the event handler
        INTERNAL_COMMAND_BUS.
            execute(InitializedCoreEvent)
            .expect("BUG: Failed to send initialized core event");

        // Update the state machine to the next state
        self.state_machine()
            .handle(&NeuralAnalyticsCoreEvents::InitializedCore);
    }

    #[action]
    pub fn headset_calibration_action(&self) {
        // Get the calibration data from the headset
        let data: HashMap<String, u8> = match INTERNAL_COMMAND_BUS.execute(ExtractCalibrationDataCommand) {
            Ok(data) => data,
            Err(err) => {
                // If the headset is disconnected, send the event
                INTERNAL_COMMAND_BUS
                    .execute(HeadsetDisconnectedEvent)
                    .expect("BUG: Failed to send headset disconnected event");

                // Then, update the state
                self.state_machine().handle(&NeuralAnalyticsCoreEvents::HeadsetDisconnected);
                return;
            }
        };

        // Foreach data vec for check if every value is under 1000, all values must be under 1000
        // If any value is over 1000, retry the calibration
        for (_, value) in data.iter() {
            if *value > 1000 {
                // TODO: Add outside event with data
                INTERNAL_COMMAND_BUS.execute(HeadsetCalibratingEvent {
                    impedance_data: data.clone(),
                }).expect("BUG: Failed to send headset calibrating event");

                // Retry the calibration
                self.state_machine().handle(&NeuralAnalyticsCoreEvents::HeadsetCalibrating);
                return;
            }
        }

        // If all values are under 1000, the calibration is successful
        self.state_machine().handle(&NeuralAnalyticsCoreEvents::HeadsetCalibrated);
    }

    #[action]
    pub fn headset_connection_action(&self) {
        match INTERNAL_COMMAND_BUS.execute(SearchHeadbandCommand) {
            Ok(_) => {
                // If the headset is connected, send the event
                INTERNAL_COMMAND_BUS
                    .execute(HeadsetConnectedEvent)
                    .expect("BUG: Failed to send headset connected event");

                // If the headset is connected, update the state
                self.state_machine().handle(&NeuralAnalyticsCoreEvents::HeadsetConnected);


            }
            Err(err) => {
                // If the headset is disconnected, send the event
                INTERNAL_COMMAND_BUS
                    .execute(HeadsetDisconnectedEvent)
                    .expect("BUG: Failed to send headset disconnected event");

                // If the headset is disconnected, update the state
                self.state_machine().handle(&NeuralAnalyticsCoreEvents::HeadsetDisconnected);
            }
        }
    }

    #[action]
    pub fn capture_data_action(&self) {
        // Get raw data from the headset
        let raw_data = match INTERNAL_COMMAND_BUS.execute(ExtractGeneralistDataCommand) {
            Ok(data) => data,
            Err(err) => {
                self.state_machine().handle(&NeuralAnalyticsCoreEvents::HeadsetDisconnected);
                return;
            }
        };

        // Process the raw data to predict the color thinking
        let predict_data: String = INTERNAL_COMMAND_BUS
            .execute(PredictColorThinkingCommand {
                headset_data: raw_data,
            })
            .expect("BUG: Failed to infer color thinking");

        // Update the light status based on the predicted color thinking
        INTERNAL_COMMAND_BUS
            .execute(UpdateLightStatusCommand {
                is_light_on: predict_data.thinking_color == "green",
            })
            .expect("CONN: Failed to update light status");

        // Send the captured data to the event handler
        INTERNAL_COMMAND_BUS
            .execute(CapturedHeadsetDataEvent {
                headset_data: raw_data,
                color_thinking: predict_data,
            })
            .expect("BUG: Failed to send captured headset data event");

        // Rerun the headset data capture
        self.state_machine().handle(&NeuralAnalyticsCoreEvents::CapturedHeadsetData);
    }
}

