use captured_headset_data_event::CapturedHeadsetDataEvent;
use headset_calibrated_event::HeadsetCalibratedEvent;
use headset_calibrating_event::HeadsetCalibratingEvent;
use headset_connected_event::HeadsetConnectedEvent;
use headset_disconnected_event::HeadsetDisconnectedEvent;
use initialized_core_event::InitializedCoreEvent;
use presage::{event_handler, CommandBus, Commands, Event};

use crate::{domain::models::event_data::EventData, INTERNAL_EVENT_HANDLER};

pub mod captured_headset_data_event;
pub mod headset_calibrated_event;
pub mod headset_calibrating_event;
pub mod headset_connected_event;
pub mod headset_disconnected_event;
pub mod initialized_core_event;


#[event_handler]
pub async fn handle_captured_headset_data_event(
    _: &mut CommandBus<presage::Error, presage::Error>, event: CapturedHeadsetDataEvent
) -> Result<presage::Commands, presage::Error> {
    unsafe {
        if INTERNAL_EVENT_HANDLER.is_some() {
            let internal_event_handler = INTERNAL_EVENT_HANDLER.unwrap();

            internal_event_handler(CapturedHeadsetDataEvent::NAME, &EventData {
                headset_data: Some(event.headset_data),
                color_thinking: Some(event.color_thinking),
                impedance_data: None,
            });
        }
    }

    Ok(Commands::new())
}

#[event_handler]
pub async fn handle_headset_calibrated_event(
    _: &mut CommandBus<presage::Error, presage::Error>, _: HeadsetCalibratedEvent
) -> Result<presage::Commands, presage::Error> {
    unsafe {
        if INTERNAL_EVENT_HANDLER.is_some() {
            let internal_event_handler = INTERNAL_EVENT_HANDLER.unwrap();

            internal_event_handler(HeadsetCalibratedEvent::NAME, &EventData {
                headset_data: None,
                color_thinking: None,
                impedance_data: None,
            });
        }
    }

    Ok(Commands::new())
}

#[event_handler]
pub async fn handle_headset_calibrating_event(
    _: &mut CommandBus<presage::Error, presage::Error>, event: HeadsetCalibratingEvent
) -> Result<presage::Commands, presage::Error> {
    unsafe {
        if INTERNAL_EVENT_HANDLER.is_some() {
            let internal_event_handler = INTERNAL_EVENT_HANDLER.unwrap();

            internal_event_handler(HeadsetCalibratingEvent::NAME, &EventData {
                headset_data: None,
                color_thinking: None,
                impedance_data: Some(event.impedance_data),
            });
        }
    }

    Ok(Commands::new())
}

#[event_handler]
pub async fn handle_headset_connected_event(
    _: &mut CommandBus<presage::Error, presage::Error>, _: HeadsetConnectedEvent
) -> Result<presage::Commands, presage::Error> {
    unsafe {
        if INTERNAL_EVENT_HANDLER.is_some() {
            let internal_event_handler = INTERNAL_EVENT_HANDLER.unwrap();

            internal_event_handler(HeadsetConnectedEvent::NAME, &EventData {
                headset_data: None,
                color_thinking: None,
                impedance_data: None,
            });
        }
    }

    Ok(Commands::new())
}

#[event_handler]
pub async fn handle_headset_disconnected_event(
    _: &mut CommandBus<presage::Error, presage::Error>, _: HeadsetDisconnectedEvent
) -> Result<presage::Commands, presage::Error> {
    unsafe {
        if INTERNAL_EVENT_HANDLER.is_some() {
            let internal_event_handler = INTERNAL_EVENT_HANDLER.unwrap();

            internal_event_handler(HeadsetDisconnectedEvent::NAME, &EventData {
                headset_data: None,
                color_thinking: None,
                impedance_data: None,
            });
        }
    }

    Ok(Commands::new())
}

#[event_handler]
pub async fn handle_initialized_core_event(
    _: &mut CommandBus<presage::Error, presage::Error>, _: InitializedCoreEvent
) -> Result<presage::Commands, presage::Error> {
    unsafe {
        if INTERNAL_EVENT_HANDLER.is_some() {
            let internal_event_handler = INTERNAL_EVENT_HANDLER.unwrap();

            internal_event_handler(InitializedCoreEvent::NAME, &EventData {
                headset_data: None,
                color_thinking: None,
                impedance_data: None,
            });
        }
    }

    Ok(Commands::new())
}
