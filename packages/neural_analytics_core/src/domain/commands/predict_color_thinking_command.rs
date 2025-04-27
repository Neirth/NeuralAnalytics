#[derive(Debug)]
pub struct PredictColorThinkingCommand;

impl presage::Command for PredictColorThinkingCommand {
    const NAME: &'static str = "predict-color-thinking";
}