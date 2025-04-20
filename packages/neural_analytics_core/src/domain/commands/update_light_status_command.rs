#[derive(Debug)]
pub struct UpdateLightStatusCommand {
    pub is_light_on: bool,
}

impl UpdateLightStatusCommand {
    pub fn new(is_light_on: bool) -> Self {
        UpdateLightStatusCommand { is_light_on }
    }
}

impl presage::Command for UpdateLightStatusCommand {
    const NAME: &'static str = "update-light-status";
}