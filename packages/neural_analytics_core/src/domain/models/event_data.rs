use std::collections::HashMap;

pub struct EventData {
    pub headset_data: Option<Vec<Vec<u8>>>,
    pub color_thinking: Option<String>,
    pub impedance_data: Option<HashMap<String, u8>>,
}