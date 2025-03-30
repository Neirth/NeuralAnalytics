#[derive(Debug)]
pub struct UpdateLightStatusCommand {
    pub is_light_on: bool,
}

impl presage::Command for UpdateLightStatusCommand {
    const NAME: &'static str = "update-light-status";
}