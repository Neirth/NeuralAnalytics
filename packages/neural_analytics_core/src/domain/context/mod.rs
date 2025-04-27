use log::info;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

use presage::{async_trait, Error, Event, EventWriter, SerializedEvent};

use crate::infrastructure::adapters::{
    input::{brainbit_headset::BrainFlowAdapter, mock_headset::MockHeadsetAdapter},
    output::tapo_smartbulb::TapoSmartBulbAdapter,
};

use super::{
    events::captured_headset_data_event::CapturedHeadsetDataEvent,
    models::event_internals::{
        ReceivedCalibrationDataEvent, ReceivedGeneralistDataEvent,
        ReceivedPredictColorThinkingDataEvent,
    },
    ports::{input::eeg_headset::EegHeadsetPort, output::smart_bulb::SmartBulbPort},
    services::model_inference_service::{ModelInferenceInterface, ModelInferenceService},
};

// Singletons for the adapters and services
static MODEL_SERVICE: OnceCell<Arc<RwLock<Box<dyn ModelInferenceInterface + Send + Sync>>>> =
    OnceCell::new();
static MOCK_HEADSET_ADAPTER: OnceCell<Arc<RwLock<Box<dyn EegHeadsetPort + Send + Sync>>>> =
    OnceCell::new();
static BRAINFLOW_ADAPTER: OnceCell<Arc<RwLock<Box<dyn EegHeadsetPort + Send + Sync>>>> =
    OnceCell::new();
static TAPO_SMARTBULB_ADAPTER: OnceCell<Arc<RwLock<Box<dyn SmartBulbPort + Send + Sync>>>> =
    OnceCell::new();

// Functions to get the singletons
fn get_model_service() -> &'static Arc<RwLock<Box<dyn ModelInferenceInterface + Send + Sync>>> {
    MODEL_SERVICE.get_or_init(|| Arc::new(RwLock::new(Box::new(ModelInferenceService::default()))))
}

fn get_mock_headset_adapter() -> &'static Arc<RwLock<Box<dyn EegHeadsetPort + Send + Sync>>> {
    MOCK_HEADSET_ADAPTER.get_or_init(|| {
        info!("Using mock adapter for EEG (real hardware not available or mock usage forced)");
        Arc::new(RwLock::new(Box::new(MockHeadsetAdapter::default())))
    })
}

fn get_brainflow_adapter() -> &'static Arc<RwLock<Box<dyn EegHeadsetPort + Send + Sync>>> {
    BRAINFLOW_ADAPTER.get_or_init(|| {
        info!("Attempting to use real BrainFlow adapter for EEG");
        Arc::new(RwLock::new(Box::new(BrainFlowAdapter::default())))
    })
}

fn get_tapo_smartbulb_adapter() -> &'static Arc<RwLock<Box<dyn SmartBulbPort + Send + Sync>>> {
    TAPO_SMARTBULB_ADAPTER
        .get_or_init(|| Arc::new(RwLock::new(Box::new(TapoSmartBulbAdapter::default()))))
}

pub(crate) struct NeuralAnalyticsContext {
    // Data Context
    pub headset_data: Option<HashMap<String, Vec<f32>>>,
    pub color_thinking: Option<String>,
    pub impedance_data: Option<HashMap<String, u16>>,

    // Ports and Adapters (referencias a los Arc<RwLock> que contienen los singletons)
    pub eeg_headset_adapter: &'static Arc<RwLock<Box<dyn EegHeadsetPort + Send + Sync>>>,
    pub smart_bulb_adapter: &'static Arc<RwLock<Box<dyn SmartBulbPort + Send + Sync>>>,

    // Services (referencia al Arc<RwLock> que contiene el singleton)
    pub model_service: &'static Arc<RwLock<Box<dyn ModelInferenceInterface + Send + Sync>>>,
}

impl Default for NeuralAnalyticsContext {
    fn default() -> Self {
        // Determine if we use the real adapter or mock based on an environment variable
        let use_mock_adapter = env::var("USE_MOCK_HEADSET")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase()
            == "true";

        // Obtain the EEG headset adapter based on the environment variable
        // If USE_MOCK_HEADSET is set to "true", use the mock adapter
        let eeg_adapter = if use_mock_adapter {
            get_mock_headset_adapter()
        } else {
            get_brainflow_adapter()
        };

        NeuralAnalyticsContext {
            // Initialize the data context
            headset_data: None,
            color_thinking: None,
            impedance_data: None,

            // Initialize the adapters con referencias a los singletons (sin clonar)
            eeg_headset_adapter: eeg_adapter,
            smart_bulb_adapter: get_tapo_smartbulb_adapter(),

            // Initialize the model service con referencia al singleton (sin clonar)
            model_service: get_model_service(),
        }
    }
}

#[async_trait]
impl EventWriter for NeuralAnalyticsContext {
    type Error = Error;

    async fn write(&mut self, event: &SerializedEvent) -> Result<(), Error> {
        if event.name() == ReceivedCalibrationDataEvent::NAME {
            let event_data = <SerializedEvent as Clone>::clone(&event)
                .deserialize::<ReceivedCalibrationDataEvent>()
                .expect("BUG: Failed to deserialize event");

            self.headset_data = None;
            self.color_thinking = None;
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

            self.color_thinking = Some(event_data.color_thinking);
            self.impedance_data = None;
        }

        Ok(())
    }
}
