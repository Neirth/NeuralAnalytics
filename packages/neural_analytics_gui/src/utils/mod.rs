use plotters::prelude::*;
use slint::{Model, ModelRc, SharedPixelBuffer};

/// Renders a chart to visualize EEG signals
/// 
/// This function takes EEG signal data and generates an image with a chart
/// similar to the one shown in the Python interface.
///
/// # Arguments
/// * `name` - Electrode name (T3, T4, O1, O2)
/// * `data` - Vector with signal values
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels
///
/// # Returns
/// * `slint::Image` - Rendered image with the chart
pub fn render_signal_plot(
    name: slint::SharedString,
    data: ModelRc<f32>,
    width: f32,
    height: f32,
) -> slint::Image {    
    let width_px = width as u32;
    let height_px = height as u32;
    
    // Create a pixel buffer for the image
    let mut pixel_buffer = SharedPixelBuffer::<slint::Rgba8Pixel>::new(width_px, height_px);
    
    {
        // Access the underlying buffer to draw with plotters
        let plotting_area = BitMapBackend::with_buffer(
            pixel_buffer.make_mut_bytes(), 
            (width_px, height_px)
        ).into_drawing_area();
        
        // Set a semi-transparent background
        plotting_area.fill(&RGBAColor(30, 30, 40, 20.0)).unwrap();
        
        // Convert ModelRc<f32> to Vec<f32> to facilitate iteration
        let data_vec: Vec<f32> = data.iter()
            .map(|value| value) // Handle possible errors
            .collect();
        
        // If there's no data, return the buffer with just the background
        if data_vec.is_empty() {
            plotting_area.present().unwrap();
        } else {
            // Determine data range for Y-axis
            let min_value = data_vec.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let max_value = data_vec.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            
            // Add margin to the range for better visualization
            let margin = (max_value - min_value).max(0.1) * 0.15;
            let y_range = (min_value - margin..max_value + margin);
            
            // Select color based on electrode
            let line_color = match name.as_str() {
                "T3" => RGBAColor(0, 255, 100, 25.0),    // Green
                "T4" => RGBAColor(255, 150, 0, 25.0),    // Orange
                "O1" => RGBAColor(50, 150, 255, 25.0),   // Blue
                "O2" => RGBAColor(255, 50, 255, 25.0),   // Magenta
                _ => RGBAColor(255, 255, 255, 25.0),     // White by default
            };
            
            // Configure and build the chart
            let mut chart = ChartBuilder::on(&plotting_area)
                .margin(10)
                .caption(
                    format!("Electrodo {}", name),
                    ("sans-serif", height_px / 20, &WHITE.mix(0.9))
                )
                .x_label_area_size(35)
                .y_label_area_size(40)
                .build_cartesian_2d(0..data_vec.len(), y_range)
                .unwrap();
            
            // Configure grid style
            chart
                .configure_mesh()
                .light_line_style(&RGBAColor(200, 200, 200, 90.0))
                .axis_style(ShapeStyle::from(&WHITE.mix(0.8)).stroke_width(2))
                .y_labels(5)
                .y_label_style(("sans-serif", height_px / 25, &WHITE.mix(0.8)))
                .x_label_style(("sans-serif", height_px / 25, &WHITE.mix(0.8)))
                .draw().unwrap();
            
            // TODO: Create and draw the point series for the EEG line
            // let line_series = LineSeries::new(
            //     data_vec.iter().enumerate().map(|(i, &v)| (i, v as f64)),
            //     line_color.stroke_width(2),
            // );
            
            // TODO: Draw the line series on the chart
            // chart.draw_series(line_series).unwrap()
            //     .label(format!("{}", name))
            //     .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &line_color));
            
            // Display legend in top-right corner
            chart.configure_series_labels()
                .background_style(&RGBAColor(30, 30, 40, 20.0))
                .border_style(&WHITE.mix(0.8))
                .position(SeriesLabelPosition::UpperRight)
                .draw().unwrap();
        }
        
        // Finalize the drawing
        plotting_area.present().unwrap();
    }
    
    // Convert buffer to a slint image with alpha channel
    slint::Image::from_rgba8(pixel_buffer)
}