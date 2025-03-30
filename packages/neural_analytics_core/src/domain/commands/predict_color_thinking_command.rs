use std::collections::HashMap;

pub struct PredictColorThinkingCommand {
    pub headset_data: HashMap<String, Vec<f32>>,
}

impl presage::Command for PredictColorThinkingCommand {
    const NAME: &'static str = "predict-color-thinking";
}