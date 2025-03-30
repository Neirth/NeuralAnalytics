#[derive(Debug)]
pub struct PredictColorThinkingCommand {
    pub headset_data: Vec<Vec<u8>>,
}

impl presage::Command for PredictColorThinkingCommand {
    const NAME: &'static str = "predict-color-thinking";
}