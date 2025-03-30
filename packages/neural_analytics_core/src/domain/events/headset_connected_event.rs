#[derive(serde::Serialize, serde::Deserialize)]
pub struct HeadsetConnectedEvent;

impl presage::Event for HeadsetConnectedEvent {
    const NAME: &'static str = "headset-connected";
}