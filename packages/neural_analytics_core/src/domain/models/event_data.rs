use std::collections::HashMap;


#[derive(Default)]
pub struct EventData {
    pub headset_data: Option<HashMap<String, Vec<f32>>>,
    pub color_thinking: Option<String>,
    pub impedance_data: Option<HashMap<String, u16>>,
}