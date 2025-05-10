use log::{debug, error};

use crate::{domain::models::event_data::EventData, INTERNAL_EVENT_HANDLER};

/// Helper function to send events to external subscribers.
/// This delegates the event to the globally registered event handler.
///
/// # Parameters
/// - `event`: Event name/identifier
/// - `data`: Event payload data
///
/// # Returns
/// - `Result<(), String>`: Success or error message
pub fn send_event(event: &String, data: &EventData) -> Result<(), String> {
    // Send the event to the event handler
    if let Some(event_handler) = unsafe { INTERNAL_EVENT_HANDLER.as_ref() } {
        let result = event_handler(event, data);
        if let Err(ref e) = result {
            error!("Error sending event '{}': {}", event, e);
        } else {
            debug!("Event '{}' sent successfully", event);
        }
        result
    } else {
        Err("BUG: Event handler not set".to_string())
    }
}
