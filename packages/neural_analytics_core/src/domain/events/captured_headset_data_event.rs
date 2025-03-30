use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CapturedHeadsetDataEvent {
    pub headset_data: HashMap<String, Vec<f32>>,
    pub color_thinking: String,
}

impl presage::Event for CapturedHeadsetDataEvent {
    const NAME: &'static str = "captured-headset-data";
}