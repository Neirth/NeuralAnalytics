use std::collections::HashMap;

use presage::{async_trait, Error, Event, EventWriter, SerializedEvent};

use super::{events::captured_headset_data_event::CapturedHeadsetDataEvent, models::event_internals::{ReceivedCalibrationDataEvent, ReceivedGeneralistDataEvent, ReceivedPredictColorThinkingDataEvent}};


#[derive(Default)]
pub(crate) struct NeuralAnalyticsContext {
    pub headset_data: Option<HashMap<String, Vec<f32>>>,
    pub color_thinking: Option<String>,
    pub impedance_data: Option<HashMap<String, u16>>,
}

#[async_trait]
impl EventWriter for NeuralAnalyticsContext {
    type Error = Error;

    async fn write(&mut self, event: &SerializedEvent) -> Result<(), Error> {
        if event.name() == ReceivedCalibrationDataEvent::NAME {
            let event_data = <SerializedEvent as Clone>::clone(&event).deserialize::<ReceivedCalibrationDataEvent>().expect("BUG: Failed to deserialize event");

            self.headset_data = None;
            self.color_thinking = None;
            self.impedance_data = Some(event_data.impedance_data);
        } else if event.name() == ReceivedGeneralistDataEvent::NAME {
            let event_data = <SerializedEvent as Clone>::clone(&event).deserialize::<CapturedHeadsetDataEvent>().expect("BUG: Failed to deserialize event");

            self.headset_data = Some(event_data.headset_data);
            self.impedance_data = None;
        } else if event.name() == ReceivedPredictColorThinkingDataEvent::NAME {
            let event_data = <SerializedEvent as Clone>::clone(&event).deserialize::<ReceivedPredictColorThinkingDataEvent>().expect("BUG: Failed to deserialize event");

            self.color_thinking = Some(event_data.color_thinking);
            self.impedance_data = None;
        }

        Ok(())
    }
}