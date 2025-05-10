use std::sync::Arc;

use log::info;
use once_cell::sync::OnceCell;
use tokio::sync::RwLock;

use crate::{domain::{ports::{input::eeg_headset::EegHeadsetPort, output::smart_bulb::SmartBulbPort}, services::model_inference_service::{ModelInferenceInterface, ModelInferenceService}}, infrastructure::adapters::{input::{brainbit_headset::BrainFlowAdapter, mock_headset::MockHeadsetAdapter}, output::tapo_smartbulb::TapoSmartBulbAdapter}};

// Singletons for the adapters and services
static MODEL_SERVICE: OnceCell<Arc<RwLock<Box<dyn ModelInferenceInterface + Send + Sync>>>> =
    OnceCell::new();
static MOCK_HEADSET_ADAPTER: OnceCell<Arc<RwLock<Box<dyn EegHeadsetPort + Send + Sync>>>> =
    OnceCell::new();
static BRAINFLOW_ADAPTER: OnceCell<Arc<RwLock<Box<dyn EegHeadsetPort + Send + Sync>>>> =
    OnceCell::new();
static TAPO_SMARTBULB_ADAPTER: OnceCell<Arc<RwLock<Box<dyn SmartBulbPort + Send + Sync>>>> =
    OnceCell::new();

/// Function to get the model service singleton
/// 
/// # Returns
/// * `&'static Arc<RwLock<Box<dyn ModelInferenceInterface + Send + Sync>>>`: A reference to the model service singleton.
pub fn get_model_service() -> &'static Arc<RwLock<Box<dyn ModelInferenceInterface + Send + Sync>>> {
    MODEL_SERVICE.get_or_init(|| Arc::new(RwLock::new(Box::new(ModelInferenceService::default()))))
}

/// Function to get the EEG headset adapter singleton
/// 
/// # Returns
/// * `&'static Arc<RwLock<Box<dyn EegHeadsetPort + Send + Sync>>>`: A reference to the EEG headset adapter singleton.
pub fn get_mock_headset_adapter() -> &'static Arc<RwLock<Box<dyn EegHeadsetPort + Send + Sync>>> {
    MOCK_HEADSET_ADAPTER.get_or_init(|| {
        info!("Using mock adapter for EEG (real hardware not available or mock usage forced)");
        Arc::new(RwLock::new(Box::new(MockHeadsetAdapter::default())))
    })
}

/// Function to get the BrainFlow EEG headset adapter singleton
/// 
/// # Returns
/// * `&'static Arc<RwLock<Box<dyn EegHeadsetPort + Send + Sync>>>`: A reference to the BrainFlow EEG headset adapter singleton.
pub fn get_brainflow_adapter() -> &'static Arc<RwLock<Box<dyn EegHeadsetPort + Send + Sync>>> {
    BRAINFLOW_ADAPTER.get_or_init(|| {
        info!("Attempting to use real BrainFlow adapter for EEG");
        Arc::new(RwLock::new(Box::new(BrainFlowAdapter::default())))
    })
}

/// Function to get the Tapo Smart Bulb adapter singleton
/// 
/// # Returns
/// * `&'static Arc<RwLock<Box<dyn SmartBulbPort + Send + Sync>>>`: A reference to the Tapo Smart Bulb adapter singleton.
pub fn get_tapo_smartbulb_adapter() -> &'static Arc<RwLock<Box<dyn SmartBulbPort + Send + Sync>>> {
    TAPO_SMARTBULB_ADAPTER
        .get_or_init(|| Arc::new(RwLock::new(Box::new(TapoSmartBulbAdapter::default()))))
}