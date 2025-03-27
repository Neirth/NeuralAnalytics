pub mod events;
pub mod state;
pub mod transition;

use statig::prelude::*;

#[derive(Default)]
pub struct NeuralStateMachine;

pub enum Event {
    InitializedCore,
    HeadsetConnected,
    HeadsetDisconnected,
    HeadsetCalibrated,
    DataCaptureStarted,
    CapturedHeadsetData,
}

#[state_machine(initial = "State::initialize_application()")]
impl NeuralStateMachine {
    #[state]
    fn initialize_application(event: &Event) -> Response<State> {
        match event {
            Event::InitializedCore => Transition(State::awaiting_headset_connection()),
            _ => Super,
        }
    }

    #[state]
    fn awaiting_headset_connection(event: &Event) -> Response<State> {
        match event {
            Event::HeadsetConnected => Transition(State::awaiting_headset_calibration()),
            Event::HeadsetDisconnected => Transition(State::awaiting_headset_connection()),
            _ => Super,
        }
    }

    #[state]
    fn awaiting_headset_calibration(event: &Event) -> Response<State> {
        match event {
            Event::HeadsetDisconnected => Transition(State::awaiting_headset_connection()),
            Event::HeadsetCalibrated => Transition(State::capturing_headset_data()),
            _ => Super,
        }
    }

    #[state]
    fn capturing_headset_data(event: &Event) -> Response<State> {
        match event {
            Event::HeadsetDisconnected => Transition(State::awaiting_headset_connection()),
            Event::CapturedHeadsetData => Transition(State::capturing_headset_data()),
            _ => Super,
        }
    }
}

/*
fn main() {
    let mut state_machine = NeuralStateMachine::default().state_machine();
    
    state_machine.handle(&Event::InitializedCore);
    state_machine.handle(&Event::HeadsetConnected);
    state_machine.handle(&Event::HeadsetCalibrated);
    state_machine.handle(&Event::CapturedHeadsetData);
    state_machine.handle(&Event::HeadsetDisconnected);
}
*/