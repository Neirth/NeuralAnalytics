use std::collections::HashMap;


#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct ReceivedGeneralistDataEvent {
    pub headset_data: HashMap<String, Vec<f32>>,
}

impl presage::Event for ReceivedGeneralistDataEvent {
    const NAME: &'static str = "received-generalist-data";
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct ReceivedCalibrationDataEvent {
    pub impedance_data: HashMap<String, u16>,
}

impl presage::Event for ReceivedCalibrationDataEvent {
    const NAME: &'static str = "received-calibration-data";
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct ReceivedPredictColorThinkingDataEvent {
    pub color_thinking: String,
}

impl presage::Event for ReceivedPredictColorThinkingDataEvent {
    const NAME: &'static str = "received-predict-color-thinking-data";
}