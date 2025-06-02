use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use singletons::{get_brainflow_adapter, get_tapo_smartbulb_adapter, get_model_service};

use presage::{async_trait, Error, Event, EventWriter, SerializedEvent};
use tokio::sync::RwLock;

use super::{
    events::captured_headset_data_event::CapturedHeadsetDataEvent,
    models::event_internals::{
        ReceivedCalibrationDataEvent, ReceivedGeneralistDataEvent,
        ReceivedPredictColorThinkingDataEvent,
    },
    ports::{input::eeg_headset::EegHeadsetPort, output::smart_bulb::SmartBulbPort},
    services::model_inference_service::ModelInferenceInterface,
};

mod singletons;

const BUFFER_SIZE: usize = 6;

pub(crate) struct NeuralAnalyticsContext {
    // Data Context
    pub headset_data: Option<HashMap<String, Vec<f32>>>,
    pub color_thinking: VecDeque<String>,
    pub impedance_data: Option<HashMap<String, u16>>,

    // Ports and Adapters (referencias a los Arc<RwLock> que contienen los singletons)
    pub eeg_headset_adapter: &'static Arc<RwLock<Box<dyn EegHeadsetPort + Send + Sync>>>,
    pub smart_bulb_adapter: &'static Arc<RwLock<Box<dyn SmartBulbPort + Send + Sync>>>,

    // Services (referencia al Arc<RwLock> que contiene el singleton)
    pub model_service: &'static Arc<RwLock<Box<dyn ModelInferenceInterface + Send + Sync>>>,
}

impl Default for NeuralAnalyticsContext {
    fn default() -> Self {
        // Obtain the EEG headset adapter based on the environment variable
        // If USE_MOCK_HEADSET is set to "true", use the mock adapter
        let eeg_adapter = get_brainflow_adapter();

        NeuralAnalyticsContext {
            // Initialize the data context
            headset_data: None,
            color_thinking: VecDeque::with_capacity(BUFFER_SIZE),
            impedance_data: None,

            // Initialize the adapters con referencias a los singletons (sin clonar)
            eeg_headset_adapter: eeg_adapter,
            smart_bulb_adapter: get_tapo_smartbulb_adapter(),

            // Initialize the model service con referencia al singleton (sin clonar)
            model_service: get_model_service(),
        }
    }
}

impl NeuralAnalyticsContext {
    /// Get the real color that the user is thinking about.
    /// 
    /// This function checks if all the colors in the `color_thinking` buffer are the same.
    /// If they are, it returns that color. Otherwise, it returns "unknown".
    /// 
    /// # Returns
    /// * `String`: The color that the user is thinking about, or "unknown" if it cannot be determined.
    pub fn get_color_thinking(&self) -> String {
        if self.color_thinking.is_empty() {
            return "unknown".to_string();
        }

        let first_color = self.color_thinking.front().unwrap();

        if self.color_thinking.iter().all(|color| color == first_color) {
           first_color.clone()
        } else {
            "unknown".to_string()
        }
    }
}

#[async_trait]
impl EventWriter for NeuralAnalyticsContext {
    type Error = Error;

    /// Write an event to the context. This function is called when an event is received.
    /// 
    /// It updates the context with the event data, depending on the type of event.
    /// 
    /// # Arguments
    /// * `event`: The serialized event to be processed.
    /// 
    /// # Returns
    /// * `Result<(), Error>`: Returns `Ok(())` if the event is processed successfully, or an error if it fails.
    async fn write(&mut self, event: &SerializedEvent) -> Result<(), Error> {
        if event.name() == ReceivedCalibrationDataEvent::NAME {
            let event_data = <SerializedEvent as Clone>::clone(&event)
                .deserialize::<ReceivedCalibrationDataEvent>()
                .expect("BUG: Failed to deserialize event");

            self.headset_data = None;
            self.impedance_data = Some(event_data.impedance_data);
        } else if event.name() == ReceivedGeneralistDataEvent::NAME {
            let event_data = <SerializedEvent as Clone>::clone(&event)
                .deserialize::<CapturedHeadsetDataEvent>()
                .expect("BUG: Failed to deserialize event");

            self.headset_data = Some(event_data.headset_data);
            self.impedance_data = None;
        } else if event.name() == ReceivedPredictColorThinkingDataEvent::NAME {
            let event_data = <SerializedEvent as Clone>::clone(&event)
                .deserialize::<ReceivedPredictColorThinkingDataEvent>()
                .expect("BUG: Failed to deserialize event");

            if self.color_thinking.len() >= BUFFER_SIZE {
                self.color_thinking.pop_front();
            }

            self.color_thinking.push_back(event_data.color_thinking);
            self.impedance_data = None;
        }

        Ok(())
    }
}
