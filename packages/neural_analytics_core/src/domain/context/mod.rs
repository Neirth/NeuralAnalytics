use std::collections::HashMap;
use std::env;
use log::{info, warn};

use presage::{async_trait, Error, Event, EventWriter, SerializedEvent};

use crate::infrastructure::adapters::{
    input::{
        brainbit_headset::BrainFlowAdapter, 
        mock_headset::MockHeadsetAdapter
    }, 
    output::tapo_smartbulb::TapoSmartBulbAdapter
};

use super::{events::captured_headset_data_event::CapturedHeadsetDataEvent, models::event_internals::{ReceivedCalibrationDataEvent, ReceivedGeneralistDataEvent, ReceivedPredictColorThinkingDataEvent}, ports::{input::eeg_headset::EegHeadsetPort, output::smart_bulb::SmartBulbPort}, services::model_inference_service::{ModelInferenceInterface, ModelInferenceService}};


pub(crate) struct NeuralAnalyticsContext {
    // Data Context
    pub headset_data: Option<HashMap<String, Vec<f32>>>,
    pub color_thinking: Option<String>,
    pub impedance_data: Option<HashMap<String, u16>>,

    // Ports and Adapters 
    pub eeg_headset_adapter: Box<dyn EegHeadsetPort>,
    pub smart_bulb_adapter: Box<dyn SmartBulbPort>,

    // Services
    pub model_service: Box<dyn ModelInferenceInterface>,
}

impl Default for NeuralAnalyticsContext {
    fn default() -> Self {
        // Determine if we use the real adapter or mock based on an environment variable
        let use_mock_adapter = env::var("USE_MOCK_HEADSET")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase() == "true";
        
        let eeg_adapter: Box<dyn EegHeadsetPort> = if use_mock_adapter {
            info!("Using mock adapter for EEG (real hardware not available or mock usage forced)");
            Box::new(MockHeadsetAdapter::default())
        } else {
            info!("Attempting to use real BrainFlow adapter for EEG");
            match BrainFlowAdapter::default() {
                adapter => {
                    if adapter.is_connected() {
                        info!("BrainFlow adapter connected successfully");
                        Box::new(adapter)
                    } else {
                        warn!("Failed to initialize BrainFlow, using mock adapter as fallback");
                        Box::new(MockHeadsetAdapter::default())
                    }
                }
            }
        };

        NeuralAnalyticsContext {
            // Initialize the data context
            headset_data: None,
            color_thinking: None,
            impedance_data: None,

            // Initialize the adapters
            eeg_headset_adapter: eeg_adapter,
            smart_bulb_adapter: Box::new(TapoSmartBulbAdapter::default()),

            // Initialize the model service
            model_service: Box::new(ModelInferenceService::default()),
        }
    }
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