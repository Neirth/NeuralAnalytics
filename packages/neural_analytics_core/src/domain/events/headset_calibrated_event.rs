#[derive(serde::Serialize, serde::Deserialize)]
pub struct HeadsetCalibratedEvent {
    pub impedance_data: Vec<u8>,
}

impl presage::Event for HeadsetCalibratedEvent {
    const NAME: &'static str = "headset-calibrated";
}