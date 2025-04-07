use plotters::{prelude::*, style::full_palette::GREY_900};
use slint::{Model, ModelRc, SharedPixelBuffer, SharedString, Image};

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
    name: SharedString,
    data: ModelRc<f32>,
    width: f32,
    height: f32,
) -> Image { 
    // Use width and height
    let width_px = width.round() as u32;
    let height_px = height.round() as u32;
    
    // INFO: Debug line
    // println!("Rendering signal plot for electrode '{}' with width: {}px, height: {}px, data points: {}", 
    //          name, width_px, height_px, data.row_count());
    
    // Create buffer of pixels
    let mut pixel_buffer = SharedPixelBuffer::<slint::Rgb8Pixel>::new(width_px, height_px);

    {
        // Create a backend for drawing in a canvas
        let root = BitMapBackend::with_buffer(pixel_buffer.make_mut_bytes(), (width_px, height_px))
            .into_drawing_area();

        // Draw the background
        root.fill(&GREY_900).unwrap();

        // Transform data to vector
        let data_vec: Vec<f32> = data.iter().collect();
        
        if data_vec.is_empty() {
            drop(root);
            return Image::from_rgb8(pixel_buffer);
        }
        
        // Normalize the data
        let (normalized_data, min_value, max_value): (Vec<f32>, f32, f32) = {
            if data_vec.len() > 1 {
                let min_value = data_vec.iter().cloned().fold(f32::INFINITY, f32::min);
                let max_value = data_vec.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                
                if (max_value - min_value).abs() < 1e-6 {
                    (data_vec.clone(), min_value, max_value)
                } else {
                    (data_vec.iter().map(|&v| {
                        2.0 * (v - min_value) / (max_value - min_value) - 1.0
                    }).collect(), min_value, max_value)
                }
            } else {
                (data_vec.clone(), 0.0, 0.0)
            }
        };

        // Draw the title
        let root_area = root.titled(
            name.as_str(),
            TextStyle::from(("Open Sans Pro", 20)).color(&WHITE)
        ).unwrap();

        // Draw the chart
        let mut chart = ChartBuilder::on(&root_area)
            .margin(10)
            .set_label_area_size(LabelAreaPosition::Left, 50)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .build_cartesian_2d(1..(normalized_data.len()), min_value..max_value)
            .unwrap();

        chart.configure_mesh()
            .axis_style(WHITE.mix(0.5))
            .x_desc("Timeseries")
            .y_desc("Signal Value")
            .x_label_style(
                ("Open Sans Pro", 15).into_text_style(&root_area).color(&WHITE)
            )
            .y_label_style(
                ("Open Sans Pro", 15).into_text_style(&root_area).color(&WHITE)
            ) // Estilo de ejes semitransparente
            .x_label_formatter(&|v| if *v % (normalized_data.len() / 5).max(1) == 0 { 
                format!("{}", v) 
            } else { 
                "".to_string() 
            })
            .y_label_formatter(&|v| format!("{:.1}", v))
            .draw()
            .unwrap();

        // Draw the data in the chart
        chart
            .draw_series(LineSeries::new(
                normalized_data.iter().enumerate().map(|(x, &y)| (x + 1, y)),
                WHITE.stroke_width(2)
            ))
            .unwrap();
        
        // Add points to every point
        if normalized_data.len() < 50 {
            chart.draw_series(PointSeries::of_element(
                normalized_data.iter()
                    .enumerate()
                    .step_by(normalized_data.len() / 5.max(1))
                    .map(|(x, &y)| (x + 1, y)),
                4,
                ShapeStyle::from(&WHITE).filled(),
                &|coord, size, style| {
                    EmptyElement::at(coord) + Circle::new((0, 0), size, style)
                },
            )).unwrap();
        }
    }

    Image::from_rgb8(pixel_buffer)
}