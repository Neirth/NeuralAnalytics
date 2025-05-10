use neural_analytics_core::{domain::events::NeuralAnalyticsEvents, initialize_core};
use neural_analytics_core::domain::models::event_data::EventData;
use utils::render_signal_plot;
use std::process::exit;
use std::sync::{Mutex, LazyLock};
use std::vec;
use slint::{ComponentHandle, ModelRc, SharedString, Weak};

pub mod utils;

slint::include_modules!();

// Global storage for our main window reference
static MAIN_WINDOW_WEAK: LazyLock<Mutex<Option<Weak<MainFrame>>>> = LazyLock::new(|| Mutex::new(None));

/// Event handler function
/// 
/// This function is called when an event occurs. It takes a string and an `EventData` struct as arguments.
/// This is part of Model View Intent (MVI) pattern. Communicates with the UI thread to update the view.
/// 
/// # Arguments
/// - `event`: A string representing the event name.
/// - `data`: An `EventData` struct containing the data associated with the event.
/// 
/// # Returns
/// - `Result<(), String>`: Returns `Ok(())` if the event is handled successfully, or an error message if it fails.
fn event_handler(event: &String, data: &EventData) -> Result<(), String> {
    // Clone the event name to avoid borrowing issues
    let event_name = event.clone();
    
    // Clone the data to avoid borrowing issues
    let impedance_data_clone = data.impedance_data.clone(); 
    let headset_data_clone = data.headset_data.clone();
    let color_thinking_clone = data.color_thinking.clone();
    
    // Execute on UI thread to avoid threading issues
    slint::invoke_from_event_loop(move || {
        let main_window = match MAIN_WINDOW_WEAK.lock().unwrap().as_ref() {
            Some(weak) => match weak.upgrade() {
                Some(win) => win,
                None => return,
            },
            None => return,
        };

        // Handle the event based on its name
        match event_name.as_str() {
            val if val == NeuralAnalyticsEvents::InitializedCoreEvent.to_string() => {
                main_window.invoke_update_current_view(SharedString::from("WelcomeUserView"));
            },
            val if val == NeuralAnalyticsEvents::HeadsetConnectedEvent.to_string() => {
                main_window.invoke_update_current_view(SharedString::from("HeadsetCalibrationView"));
            },
            val if val == NeuralAnalyticsEvents::HeadsetDisconnectedEvent.to_string() => {
                main_window.invoke_update_current_view(SharedString::from("WelcomeUserView"));
            },
            val if val == NeuralAnalyticsEvents::HeadsetCalibratingEvent.to_string() => {
                if let Some(impedance_data) = &impedance_data_clone {
                    main_window.invoke_update_electrode_status(
                        impedance_data.get("T3").cloned().unwrap_or(0) as i32,
                        impedance_data.get("T4").cloned().unwrap_or(0) as i32,
                        impedance_data.get("O1").cloned().unwrap_or(0) as i32,
                        impedance_data.get("O2").cloned().unwrap_or(0) as i32,
                    );
                }
            },
            val if val == NeuralAnalyticsEvents::HeadsetCalibratedEvent.to_string() => {
                main_window.invoke_update_current_view(SharedString::from("DataCapturerView"));
            },
            val if val == NeuralAnalyticsEvents::CapturedHeadsetDataEvent.to_string() => {
                if let Some(headset_data) = &headset_data_clone {
                    main_window.invoke_update_headset_data(
                        ModelRc::from(&headset_data.get("T3").cloned().unwrap_or(vec![0.0])[..]),
                        ModelRc::from(&headset_data.get("T4").cloned().unwrap_or(vec![0.0])[..]),
                        ModelRc::from(&headset_data.get("O1").cloned().unwrap_or(vec![0.0])[..]),
                        ModelRc::from(&headset_data.get("O2").cloned().unwrap_or(vec![0.0])[..]),
                    );
                }

                if let Some(color_thinking) = &color_thinking_clone {
                    main_window.invoke_update_thinking_color(
                        SharedString::from(color_thinking),
                    );
                }
            },
            _ => {}
        }
    }).map_err(|e| format!("BUG: UI thread error; {:?}", e))?;
    
    Ok(())
}

/// Main function
/// 
/// This is the entry point of the application. It creates the main window and initializes the core.
/// It also sets the initial view and runs the application.
#[tokio::main]
async fn main() {
    env_logger::init();

    let main_window = MainFrame::new();

    if main_window.is_ok() {
        // Initialize the main window
        let main_window = main_window.unwrap();
        
        // Store a weak reference to our window globally
        *MAIN_WINDOW_WEAK.lock().unwrap() = Some(main_window.as_weak());
        
        // Set up the signal plot rendering
        main_window.on_render_signal_plot(render_signal_plot);

        // Set up the event handler
        main_window.on_start_core_process(|| {
            tokio::spawn(async {
                // Initialize the core with the event handler
                if let Err(e) = initialize_core(event_handler).await {
                    panic!("BUG: Failed to initialize core: {}", e);
                }
            });
            true
        });

        // Set initial view
        main_window.invoke_update_current_view(SharedString::from("LoadingApplicationView"));

        main_window.window().on_close_requested(|| {
            exit(0);
        });
        
        // Run the application
        main_window.run().unwrap();
    } else {
        panic!("BUG: Failed to create the main window.");
    }
}