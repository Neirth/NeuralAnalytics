pub mod libs;

slint::include_modules!();

fn main() {
    let main_window = MainWindow::new();

    if main_window.is_ok() {
        let main_window = main_window.unwrap();

        // // Callbacks for get plots 
        // // TODO: Need to be connected with the events from core
        // main_window.on_render_t3_plot(f);
        // main_window.on_render_t4_plot(f);
        // main_window.on_render_o1_plot(f);
        // main_window.on_render_o2_plot(f);

        // // Callbacks for get thinking color
        // // TODO: Need to be connected with the events from core
        // main_window.on_thinking_color(f);

        // Function to change view in Slint from Core Response
        main_window.invoke_update_current_view("WelcomeUserView".into());

        // Function to update impedance status from Core Responses
        main_window.invoke_update_electrode_status(0, 0, 0, 0);

        // Run the view
        main_window.run().unwrap();
    } else {
        eprintln!("Failed to create the main window.");
    }
}