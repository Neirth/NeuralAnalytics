#[derive(Debug)]
pub struct InitializeHardwarePartsCommand;

impl presage::Command for InitializeHardwarePartsCommand {
    const NAME: &'static str = "initialize-hardware-parts";
}