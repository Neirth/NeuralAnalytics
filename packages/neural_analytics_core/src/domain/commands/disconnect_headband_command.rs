#[derive(Debug)]
pub struct DisconnectHeadbandCommand;

impl presage::Command for DisconnectHeadbandCommand {
    const NAME: &'static str = "disconnect-headband";
}