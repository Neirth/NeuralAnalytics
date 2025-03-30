#[derive(serde::Serialize, serde::Deserialize)]
pub struct HeadsetDisconnectedEvent;

impl presage::Event for HeadsetDisconnectedEvent {
    const NAME: &'static str = "headset-disconnected";
}