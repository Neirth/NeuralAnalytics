use plotters::{prelude::*, style::full_palette::GREY_900};
use slint::{Image, Model, ModelRc, SharedPixelBuffer, SharedString};

/// Renders a chart to visualize EEG signals
///
/// This function takes EEG signal data and generates an image with a chart
/// similar to the one shown in the Python interface.
/// Raw EEG data is normalized per-channel using z-score for display.
///
/// # Arguments
/// * `name` - Electrode name (T3, T4, O1, O2)
/// * `data` - Vector with raw EEG signal values
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

        // Apply z-score normalization per-channel for visualization
        // This centers data around 0 with std ~1
        let mean: f32 = data_vec.iter().sum::<f32>() / data_vec.len() as f32;
        let variance: f32 = data_vec.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / data_vec.len() as f32;
        let std = variance.sqrt().max(1e-6); // Avoid division by zero
        
        let normalized_data: Vec<f32> = data_vec.iter().map(|x| (x - mean) / std).collect();

        // Calculate current min and max of normalized values
        let min_value = normalized_data
            .iter()
            .cloned()
            .fold(f32::INFINITY, f32::min);
        let max_value = normalized_data
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);

        // For z-score data, use symmetric range around 0
        let abs_max = max_value.abs().max(min_value.abs());
        let margin = 0.2; // Add 20% margin
        
        // Set range to be symmetric around 0
        let (final_min, final_max) = if abs_max < 0.5 {
            (-1.5, 1.5) // Minimum range for very flat signals
        } else {
            (-(abs_max + margin), abs_max + margin)
        };

        // Draw the title
        let root_area = root
            .titled(
                name.as_str(),
                TextStyle::from(("Open Sans Pro", 20)).color(&WHITE),
            )
            .unwrap();

        // Draw the chart
        let mut chart = ChartBuilder::on(&root_area)
            .margin(10)
            .set_label_area_size(LabelAreaPosition::Left, 50)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .build_cartesian_2d(1..(normalized_data.len()), final_min..final_max)
            .unwrap();

        chart
            .configure_mesh()
            .axis_style(WHITE.mix(0.5))
            .x_desc("Timeseries")
            .y_desc("Z-Score")
            .x_label_style(
                ("Open Sans Pro", 15)
                    .into_text_style(&root_area)
                    .color(&WHITE),
            )
            .y_label_style(
                ("Open Sans Pro", 15)
                    .into_text_style(&root_area)
                    .color(&WHITE),
            )
            .x_label_formatter(&|v| {
                let mod_value = (normalized_data.len() / 5).max(1);
                if *v % mod_value == 0 {
                    format!("{}", v)
                } else {
                    "".to_string()
                }
            })
            .y_label_formatter(&|v| format!("{:.1}", v))
            .draw()
            .unwrap();

        // Draw the data in the chart
        chart
            .draw_series(LineSeries::new(
                normalized_data.iter().enumerate().map(|(x, &y)| (x + 1, y)),
                WHITE.stroke_width(2),
            ))
            .unwrap();

        // Add points to every point
        if normalized_data.len() < 50 {
            // Calculamos step_size asegurándonos de que nunca sea 0
            let step_size = (normalized_data.len() / 5).max(1);

            chart
                .draw_series(PointSeries::of_element(
                    normalized_data
                        .iter()
                        .enumerate()
                        .step_by(step_size)
                        .map(|(x, &y)| (x + 1, y)),
                    4,
                    ShapeStyle::from(&WHITE).filled(),
                    &|coord, size, style| {
                        EmptyElement::at(coord) + Circle::new((0, 0), size, style)
                    },
                ))
                .unwrap();
        }
    }

    Image::from_rgb8(pixel_buffer)
}
