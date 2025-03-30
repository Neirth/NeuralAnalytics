#[derive(Debug)]
pub struct ExtractCalibrationDataCommand;

impl presage::Command for ExtractCalibrationDataCommand {
    const NAME: &'static str = "extract-calibration-data";
}