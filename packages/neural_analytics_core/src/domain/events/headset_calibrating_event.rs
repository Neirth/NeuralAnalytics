use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HeadsetCalibratingEvent {
    pub impedance_data: HashMap<String, u16>,
}

impl presage::Event for HeadsetCalibratingEvent {
    const NAME: &'static str = "headset-calibrating";
}