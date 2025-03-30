#[derive(serde::Serialize, serde::Deserialize)]
pub struct CapturedHeadsetDataEvent {
    pub headset_data: Vec<Vec<u8>>,
    pub color_thinking: String,
}

impl presage::Event for CapturedHeadsetDataEvent {
    const NAME: &'static str = "captured-headset-data";
}