use presage::Event;

pub mod captured_headset_data_event;
pub mod headset_calibrated_event;
pub mod headset_calibrating_event;
pub mod headset_connected_event;
pub mod headset_disconnected_event;
pub mod initialized_core_event;

#[derive(Debug)]
pub enum NeuralAnalyticsEvents {
    HeadsetConnectedEvent,
    HeadsetDisconnectedEvent,
    HeadsetCalibratingEvent,
    HeadsetCalibratedEvent,
    CapturedHeadsetDataEvent,
    InitializedCoreEvent,
}

impl NeuralAnalyticsEvents {
    pub fn to_string(&self) -> String {
        match self {
            NeuralAnalyticsEvents::HeadsetConnectedEvent => headset_connected_event::HeadsetConnectedEvent::NAME.to_string(),
            NeuralAnalyticsEvents::HeadsetDisconnectedEvent => headset_disconnected_event::HeadsetDisconnectedEvent::NAME.to_string(),
            NeuralAnalyticsEvents::HeadsetCalibratingEvent => headset_calibrating_event::HeadsetCalibratingEvent::NAME.to_string(),
            NeuralAnalyticsEvents::HeadsetCalibratedEvent => headset_calibrated_event::HeadsetCalibratedEvent::NAME.to_string(),
            NeuralAnalyticsEvents::CapturedHeadsetDataEvent => captured_headset_data_event::CapturedHeadsetDataEvent::NAME.to_string(),
            NeuralAnalyticsEvents::InitializedCoreEvent => initialized_core_event::InitializedCoreEvent::NAME.to_string(),
        }
    }

    pub fn from_string(event_name: &str) -> Option<Self> {
        match event_name {
            headset_connected_event::HeadsetConnectedEvent::NAME => Some(NeuralAnalyticsEvents::HeadsetConnectedEvent),
            headset_disconnected_event::HeadsetDisconnectedEvent::NAME => Some(NeuralAnalyticsEvents::HeadsetDisconnectedEvent),
            headset_calibrating_event::HeadsetCalibratingEvent::NAME => Some(NeuralAnalyticsEvents::HeadsetCalibratingEvent),
            headset_calibrated_event::HeadsetCalibratedEvent::NAME => Some(NeuralAnalyticsEvents::HeadsetCalibratedEvent),
            captured_headset_data_event::CapturedHeadsetDataEvent::NAME => Some(NeuralAnalyticsEvents::CapturedHeadsetDataEvent),
            initialized_core_event::InitializedCoreEvent::NAME => Some(NeuralAnalyticsEvents::InitializedCoreEvent),
            _ => None,
        }
    }
}