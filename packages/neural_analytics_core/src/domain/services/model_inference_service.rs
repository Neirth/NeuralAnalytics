use log::{info, warn};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tract_onnx::prelude::*;

// Trait that defines the interface for the inference service
pub trait ModelInferenceInterface: Send + Sync + 'static {
    /// Predicts the color the user is thinking based on EEG data
    fn predict_color(&self, eeg_data: &HashMap<String, Vec<f32>>) -> Result<String, String>;

    /// Checks if the model is loaded and ready for predictions
    fn is_model_loaded(&self) -> bool;
}

pub struct ModelInferenceService {
    // The ONNX model loaded using tract-onnx
    model: Option<Arc<RunnableModel<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>>,
    // Path to the model file
    model_path: String,
}

impl Default for ModelInferenceService {
    fn default() -> Self {
        // Define the default path to the model
        let model_path = "assets/neural_analytics.onnx".to_string();
        let mut service = Self {
            model: None,
            model_path,
        };

        // Try to load the model automatically
        match service.load_model() {
            Ok(_) => info!("ONNX model successfully loaded with tract-onnx"),
            Err(e) => warn!("Could not load the model automatically: {}", e),
        }

        service
    }
}

impl Drop for ModelInferenceService {
    fn drop(&mut self) {
        // Explicitly release resources if necessary
        if self.model.is_some() {
            info!("Releasing tract-onnx model resources");
            self.model = None;
        }
    }
}

impl ModelInferenceService {
    // Custom constructor if we need a different path
    pub fn new(model_path: &str) -> Self {
        let mut service = Self {
            model: None,
            model_path: model_path.to_string(),
        };

        // Try to load the model
        match service.load_model() {
            Ok(_) => info!("ONNX model successfully loaded from: {}", model_path),
            Err(e) => warn!("Could not load the model from {}: {}", model_path, e),
        }

        service
    }

    /// Loads the ONNX model from the specified path using tract-onnx
    pub fn load_model(&mut self) -> Result<(), String> {
        let path = Path::new(&self.model_path);

        if !path.exists() {
            return Err(format!(
                "Model file does not exist at path: {}",
                self.model_path
            ));
        }

        // Load model with tract-onnx
        match tract_onnx::onnx()
            .model_for_path(&self.model_path)
            .map_err(|e| format!("Error loading the model: {}", e))
            .and_then(|model| {
                model
                    .into_optimized()
                    .map_err(|e| format!("Error optimizing the model: {}", e))
            })
            .and_then(|model| {
                model
                    .into_runnable()
                    .map_err(|e| format!("Error creating runnable model: {}", e))
            }) {
            Ok(model) => {
                self.model = Some(Arc::new(model));
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Preprocesses the EEG data before passing it to the model
    /// This function implements the same preprocessing used in training
    /// and formats the data into the expected shape [batch_size, 62, 4]
    fn preprocess_data(&self, eeg_data: &HashMap<String, Vec<f32>>) -> Result<Vec<f32>, String> {
        // Check that the required channels are present
        let required_channels = ["T3", "T4", "O1", "O2"];
        for channel in required_channels.iter() {
            if !eeg_data.contains_key(*channel) {
                return Err(format!(
                    "Required channel '{}' not found in EEG data",
                    channel
                ));
            }
        }

        // Process each channel to obtain 62 normalized values per channel
        // Then we organize the data in the format expected by the model [batch_size, 62, 4]
        let expected_samples = 62; // The model expects 62 temporal samples
        let mut normalized_channels = Vec::new();

        for channel in required_channels.iter() {
            let channel_data = eeg_data.get(*channel).unwrap();

            if channel_data.is_empty() {
                return Err(format!("Channel '{}' has no data", channel));
            }

            // Tomamos todos los valores disponibles
            let mut channel_values = channel_data.clone();

            // Apply normalization similar to that used in training
            let mean = channel_values.iter().sum::<f32>() / channel_values.len() as f32;
            let variance = channel_values
                .iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f32>()
                / channel_values.len() as f32;
            let std_dev = variance.sqrt();

            // Normalize the channel data
            for value in &mut channel_values {
                *value = (*value - mean) / (std_dev + 1e-6);
            }

            // Resize or truncate to exactly 62 elements
            if channel_values.len() < expected_samples {
                // If there are fewer than 62 samples, we repeat the last one
                let last_value = *channel_values.last().unwrap_or(&0.0);
                channel_values.resize(expected_samples, last_value);
            } else if channel_values.len() > expected_samples {
                // If there are more than 62 samples, we keep the first 62
                channel_values.truncate(expected_samples);
            }

            // Store the normalized and resized channel data
            normalized_channels.push(channel_values);
        }

        // Now we have 4 channels with 62 values each
        // We organize them into a flat vector that will later be reshaped as [1, 62, 4]
        let mut processed_data = Vec::with_capacity(4 * expected_samples);

        // IMPORTANT: The LSTM model expects data organized as [batch_size, seq_length, input_size]
        // where seq_length=62 (temporal points) and input_size=4 (channels)
        // Each temporal entry must contain values from all channels for that time point.

        // The correct way to organize the data is:
        // [T3_0, T4_0, O1_0, O2_0, T3_1, T4_1, O1_1, O2_1, ..., T3_18, T4_18, O1_18, O2_18]
        for i in 0..expected_samples {
            for j in 0..normalized_channels.len() {
                processed_data.push(normalized_channels[j][i]);
            }
        }

        // Log information about the processed data
        info!(
            "Preprocessed data: {} channels x {} samples = {} elements",
            required_channels.len(),
            expected_samples,
            processed_data.len()
        );

        Ok(processed_data)
    }
}

impl ModelInferenceInterface for ModelInferenceService {
    fn predict_color(&self, eeg_data: &HashMap<String, Vec<f32>>) -> Result<String, String> {
        // Check that the model is loaded
        let model = match &self.model {
            Some(model) => model.clone(),
            None => return Err("Model is not loaded. Call load_model first.".to_string()),
        };

        // Preprocess the data
        let processed_data = self.preprocess_data(eeg_data)?;

        // Log the actual length of the processed data
        info!("Processed data length: {}", processed_data.len());

        // We verify that we have exactly 62*4 = 76 elements (62 temporal samples, 4 channels)
        let expected_elements = 62 * 4;
        if processed_data.len() != expected_elements {
            return Err(format!(
                "Processed data has unexpected length: {} (expected {})",
                processed_data.len(),
                expected_elements
            ));
        }

        // Convert processed data to tract tensor
        let batch_size = 1; // We process one example at a time

        info!(
            "Creating tensor with shape [batch_size={}, 62, 4]",
            batch_size
        );

        // Create a tensor with the correct shape [batch_size, 62, 4]
        let input_tensor =
            tract_ndarray::Array3::from_shape_vec((batch_size, 62, 4), processed_data.clone())
                .map_err(|e| format!("Error creating input tensor: {}", e))?
                .into_arc_tensor();

        // Perform inference with tract-onnx
        let outputs = match model.run(tvec!(tract_onnx::prelude::TValue::Const(input_tensor))) {
            Ok(outputs) => outputs,
            Err(e) => return Err(format!("Error during inference: {}", e)),
        };

        // Get the output tensor
        if outputs.is_empty() {
            return Err("No outputs returned from model".to_string());
        }

        // Convertir el tensor de salida a un vector
        let output_tensor = &outputs[0];
        let output_view = output_tensor
            .to_array_view::<f32>()
            .map_err(|e| format!("Error converting output to array: {}", e))?;

        // Aplicar softmax manualmente si es necesario
        let mut output_vec = output_view.iter().cloned().collect::<Vec<f32>>();

        // Aplicar softmax (esto es opcional si la red ya lo hace)
        let mut max_val = output_vec[0];
        for &val in &output_vec {
            if val > max_val {
                max_val = val;
            }
        }

        // Calcular exp(x_i - max) para cada elemento y la suma
        let mut sum = 0.0;
        for val in &mut output_vec {
            *val = (*val - max_val).exp();
            sum += *val;
        }

        // Normalizar para obtener probabilidades
        for val in &mut output_vec {
            *val /= sum;
        }

        // Map indices to colors (adjust according to model classes)
        let color_map = ["red", "green", "trash"];

        if output_vec.is_empty() {
            return Err("No probabilities obtained from the model".to_string());
        }

        // Find the color with the highest probability
        let mut max_prob = output_vec[0];
        let mut max_idx = 0;

        for (i, &prob) in output_vec.iter().enumerate() {
            if prob > max_prob {
                max_prob = prob;
                max_idx = i;
            }
        }

        // Check that the index is valid
        if max_idx >= color_map.len() {
            return Err(format!("Prediction index out of range: {}", max_idx));
        }

        // Return the predicted color
        Ok(color_map[max_idx].to_string())
    }

    fn is_model_loaded(&self) -> bool {
        self.model.is_some()
    }
}
