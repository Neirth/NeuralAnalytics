#[derive(Debug)]
pub struct ExtractGeneralistDataCommand;

impl presage::Command for ExtractGeneralistDataCommand {
    const NAME: &'static str = "extract-generalist-data";
}