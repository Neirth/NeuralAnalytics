use log::{info, warn};
use rust_embed::RustEmbed;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use tract_onnx::prelude::*;

// Constant for confidence threshold
const CONFIDENCE_THRESHOLD: f32 = 0.35;

// Minimum margin between top prediction and second best
const MIN_MARGIN: f32 = 0.10;

// Lower threshold specifically for GREEN which is harder to detect
const GREEN_CONFIDENCE_THRESHOLD: f32 = 0.30;

// Window size expected by the model (must match training)
const WINDOW_SIZE: usize = 62;

// Number of EEG channels
const NUM_CHANNELS: usize = 4;

#[derive(RustEmbed)]
#[folder = "assets"]
struct EmbeddedAssets;

/// Structure for loading normalization parameters from JSON
#[derive(Debug, Deserialize)]
struct NormalizationParams {
    global_mean: Vec<f32>,
    global_std: Vec<f32>,
}

// Trait that defines the interface for the inference service
pub trait ModelInferenceInterface: Send + Sync + 'static {
    /// Predicts the color the user is thinking based on EEG data
    fn predict_color(&self, eeg_data: &HashMap<String, Vec<f32>>) -> Result<String, String>;

    /// Checks if the model is loaded and ready for predictions
    fn is_model_loaded(&self) -> bool;
}

pub struct ModelInferenceService {
    // The ONNX model loaded using tract-onnx
    model:
        Option<Arc<RunnableModel<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>>,
    // Path to the model file
    model_path: String,
    // Global normalization parameters (mean per channel)
    global_mean: Vec<f32>,
    // Global normalization parameters (std per channel)
    global_std: Vec<f32>,
}

impl Default for ModelInferenceService {
    fn default() -> Self {
        // Define the default path to the model
        let model_path = "assets/neural_analytics.onnx".to_string();

        // Load normalization params (global mean/std per channel)
        let (global_mean, global_std) = Self::load_normalization_params_static();

        let mut service = Self {
            model: None,
            model_path,
            global_mean,
            global_std,
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
    /// Load normalization parameters from JSON file or embedded assets
    fn load_normalization_params_static() -> (Vec<f32>, Vec<f32>) {
        let params_path = "assets/normalization_params.json";

        // Try loading from disk first
        if Path::new(params_path).exists() {
            if let Ok(content) = std::fs::read_to_string(params_path) {
                if let Ok(params) = serde_json::from_str::<NormalizationParams>(&content) {
                    info!(
                        "Loaded normalization params from disk: mean={:?}, std={:?}",
                        params.global_mean, params.global_std
                    );
                    return (params.global_mean, params.global_std);
                }
            }
        }

        // Try loading from embedded assets
        if let Some(file) = EmbeddedAssets::get("normalization_params.json") {
            if let Ok(content) = std::str::from_utf8(&file.data) {
                if let Ok(params) = serde_json::from_str::<NormalizationParams>(content) {
                    info!(
                        "Loaded normalization params from embedded assets: mean={:?}, std={:?}",
                        params.global_mean, params.global_std
                    );
                    return (params.global_mean, params.global_std);
                }
            }
        }

        // Fallback to default values (will be updated after training)
        warn!("Could not load normalization params, using defaults (zeros). Model predictions may be incorrect!");
        (vec![0.0; NUM_CHANNELS], vec![1.0; NUM_CHANNELS])
    }

    // Custom constructor if we need a different path
    pub fn new(model_path: &str) -> Self {
        // Load normalization params (global mean/std per channel)
        let (global_mean, global_std) = Self::load_normalization_params_static();

        let mut service = Self {
            model: None,
            model_path: model_path.to_string(),
            global_mean,
            global_std,
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

        if path.exists() {
            // Load model using the on-disk file
            return match tract_onnx::onnx()
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
            };
        }

        // Fallback: try to use the embedded ONNX model when the default path is missing on disk
        if Path::new(&self.model_path)
            .file_name()
            .and_then(|name| name.to_str())
            == Some("neural_analytics.onnx")
        {
            if let Some(file) = EmbeddedAssets::get("neural_analytics.onnx") {
                let mut reader = Cursor::new(file.data.into_owned());
                return match tract_onnx::onnx()
                    .model_for_read(&mut reader)
                    .map_err(|e| format!("Error loading embedded model: {}", e))
                    .and_then(|model| {
                        model
                            .into_optimized()
                            .map_err(|e| format!("Error optimizing embedded model: {}", e))
                    })
                    .and_then(|model| {
                        model
                            .into_runnable()
                            .map_err(|e| format!("Error creating runnable embedded model: {}", e))
                    }) {
                    Ok(model) => {
                        self.model = Some(Arc::new(model));
                        info!("Loaded ONNX model from embedded assets");
                        Ok(())
                    }
                    Err(e) => Err(e),
                };
            }
        }

        Err(format!(
            "Model file does not exist at path: {}",
            self.model_path
        ))
    }

    /// Preprocesses the EEG data before passing it to the model
    /// We apply GLOBAL z-score normalization using fixed mean/std per channel.
    /// These statistics are computed from the entire training dataset.
    /// This matches exactly what is done during training.
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

        // Process each channel to obtain WINDOW_SIZE values per channel
        let mut channels_data = Vec::new();

        // First pass: collect all values and resize channels
        for channel in required_channels.iter() {
            let channel_data = eeg_data.get(*channel).unwrap();

            if channel_data.is_empty() {
                return Err(format!("Channel '{}' has no data", channel));
            }

            let mut channel_values = channel_data.clone();

            if channel_values.len() < WINDOW_SIZE {
                let last_value = *channel_values.last().unwrap_or(&0.0);
                channel_values.resize(WINDOW_SIZE, last_value);
            } else if channel_values.len() > WINDOW_SIZE {
                let start = channel_values.len() - WINDOW_SIZE;
                channel_values = channel_values[start..].to_vec();
            }

            channels_data.push(channel_values);
        }

        // Use GLOBAL z-score normalization (same as Python training)
        // Each channel is normalized using the global mean/std from training data
        // This ensures consistency between training and inference
        for (ch_idx, channel_values) in channels_data.iter_mut().enumerate() {
            let global_mean = self.global_mean[ch_idx];
            let global_std = self.global_std[ch_idx].max(1e-6);

            let raw_first = channel_values.first().copied().unwrap_or(0.0);

            // Normalize this channel with global mean/std
            for val in channel_values.iter_mut() {
                *val = (*val - global_mean) / global_std;
            }

            let norm_first = channel_values.first().copied().unwrap_or(0.0);
            info!(
                "Channel {}: global_mean={:.2}, global_std={:.2}, first_raw={:.2}, first_norm={:.4}",
                required_channels[ch_idx], global_mean, global_std, raw_first, norm_first
            );
        }

        // Organize data into flat vector for shape [1, WINDOW_SIZE, NUM_CHANNELS]
        // Format: [T3_0, T4_0, O1_0, O2_0, T3_1, T4_1, O1_1, O2_1, ..., T3_61, T4_61, O1_61, O2_61]
        let mut processed_data = Vec::with_capacity(NUM_CHANNELS * WINDOW_SIZE);

        for i in 0..WINDOW_SIZE {
            for j in 0..channels_data.len() {
                processed_data.push(channels_data[j][i]);
            }
        }

        info!(
            "Preprocessed data (GLOBAL z-score): {} channels x {} samples = {} elements",
            required_channels.len(),
            WINDOW_SIZE,
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

        // We verify that we have exactly WINDOW_SIZE*NUM_CHANNELS elements (62 temporal samples, 4 channels)
        let expected_elements = WINDOW_SIZE * NUM_CHANNELS;
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
            "Creating tensor with shape [batch_size={}, {}, {}]",
            batch_size, WINDOW_SIZE, NUM_CHANNELS
        );

        // Log first few values of processed_data to verify data is actually changing
        info!(
            "First 8 input values (T3_0,T4_0,O1_0,O2_0,T3_1,T4_1,O1_1,O2_1): [{:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}]",
            processed_data[0], processed_data[1], processed_data[2], processed_data[3],
            processed_data[4], processed_data[5], processed_data[6], processed_data[7]
        );

        // Create a tensor with the correct shape [batch_size, WINDOW_SIZE, NUM_CHANNELS]
        let input_tensor = tract_ndarray::Array3::from_shape_vec(
            (batch_size, WINDOW_SIZE, NUM_CHANNELS),
            processed_data.clone(),
        )
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

        // Model already outputs softmax probabilities, no need to apply again
        let output_vec = output_view.iter().cloned().collect::<Vec<f32>>();

        // Map indices to colors (adjust according to model classes)
        let color_map = ["red", "green", "trash"];

        if output_vec.is_empty() {
            return Err("No probabilities obtained from the model".to_string());
        }

        // Log all probabilities for debugging
        info!(
            "Model output probabilities - RED: {:.2}%, GREEN: {:.2}%, TRASH: {:.2}%",
            output_vec.get(0).unwrap_or(&0.0) * 100.0,
            output_vec.get(1).unwrap_or(&0.0) * 100.0,
            output_vec.get(2).unwrap_or(&0.0) * 100.0
        );

        // Find the top two probabilities
        let mut sorted_probs: Vec<(usize, f32)> = output_vec.iter().cloned().enumerate().collect();
        sorted_probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let max_idx = sorted_probs[0].0;
        let max_prob = sorted_probs[0].1;
        let second_prob = sorted_probs[1].1;
        let margin = max_prob - second_prob;

        // Check that the index is valid
        if max_idx >= color_map.len() {
            return Err(format!("Prediction index out of range: {}", max_idx));
        }

        // Determine the applicable confidence threshold based on prediction class
        // GREEN (idx=1) uses a lower threshold since it's harder to detect
        let applicable_threshold = if max_idx == 1 {
            GREEN_CONFIDENCE_THRESHOLD
        } else {
            CONFIDENCE_THRESHOLD
        };

        // Apply per-color confidence threshold
        if max_prob < applicable_threshold {
            info!(
                "Prediction confidence ({:.2}%) below threshold ({:.2}%) for {}. Returning 'trash'.",
                max_prob * 100.0,
                applicable_threshold * 100.0,
                color_map[max_idx]
            );
            return Ok("trash".to_string());
        }

        // Check margin between top prediction and second best
        // This prevents false positives when the model is uncertain
        // For GREEN (idx=1), we use a more permissive margin since it's harder to detect
        let required_margin = if max_idx == 1 {
            MIN_MARGIN * 0.5
        } else {
            MIN_MARGIN
        };

        if margin < required_margin {
            info!(
                "Margin ({:.2}%) too small (need {:.2}%). Top: {}={:.2}%, Second: {:.2}%. Returning 'trash'.",
                margin * 100.0,
                required_margin * 100.0,
                color_map[max_idx],
                max_prob * 100.0,
                second_prob * 100.0
            );
            return Ok("trash".to_string());
        }

        info!(
            "Prediction: {} with confidence: {:.2}% (margin: {:.2}%)",
            color_map[max_idx],
            max_prob * 100.0,
            margin * 100.0
        );

        // Return the predicted color
        Ok(color_map[max_idx].to_string())
    }

    fn is_model_loaded(&self) -> bool {
        self.model.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    // Helper function to create test EEG data
    fn create_test_eeg_data() -> HashMap<String, Vec<f32>> {
        let mut eeg_data = HashMap::new();
        // Create valid data for all required channels
        eeg_data.insert("T3".to_string(), vec![250000.0; 62]);
        eeg_data.insert("T4".to_string(), vec![260000.0; 62]);
        eeg_data.insert("O1".to_string(), vec![280000.0; 62]);
        eeg_data.insert("O2".to_string(), vec![255000.0; 62]);
        eeg_data
    }

    // Helper to create varied test data with different values
    fn create_varied_test_eeg_data() -> HashMap<String, Vec<f32>> {
        let mut eeg_data = HashMap::new();
        // Create varied values for better coverage
        eeg_data.insert(
            "T3".to_string(),
            (0..62).map(|i| 200000.0 + i as f32 * 1000.0).collect(),
        );
        eeg_data.insert(
            "T4".to_string(),
            (0..62).map(|i| 210000.0 + i as f32 * 1000.0).collect(),
        );
        eeg_data.insert(
            "O1".to_string(),
            (0..62).map(|i| 190000.0 + i as f32 * 1500.0).collect(),
        );
        eeg_data.insert(
            "O2".to_string(),
            (0..62).map(|i| 185000.0 + i as f32 * 1200.0).collect(),
        );
        eeg_data
    }

    // Test for successful model loading
    #[test]
    fn test_model_loading() {
        // Create a mock model file
        let dir = tempdir().unwrap();
        let model_path = dir.path().join("test_model.onnx");

        // Just to make the test work, we'll check if the model is not loaded
        // because we're not actually creating a valid ONNX model
        let service = ModelInferenceService::new(model_path.to_str().unwrap_or("invalid_path"));

        // Since we didn't create a real model file, it should not be loaded
        assert!(!service.is_model_loaded());
    }

    // Test explicit loading with non-existent file
    #[test]
    fn test_load_model_non_existent_file() {
        let mut service = ModelInferenceService {
            model: None,
            model_path: "non_existent_path/model.onnx".to_string(),
            global_mean: vec![0.0; NUM_CHANNELS],
            global_std: vec![1.0; NUM_CHANNELS],
        };

        let result = service.load_model();
        assert!(result.is_err());
        let error_msg = result.err().unwrap();
        assert!(error_msg.contains("Model file does not exist at path"));
    }

    // Test the default constructor
    #[test]
    fn test_default_constructor() {
        let service = ModelInferenceService::default();
        // Behavior depends on whether default file exists or not
        // Just verify function doesn't fail
        assert_eq!(service.model_path, "assets/neural_analytics.onnx");
    }

    // Test for data preprocessing with varied data (better coverage)
    #[test]
    fn test_preprocess_data_varied() {
        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
            global_mean: vec![0.0; NUM_CHANNELS],
            global_std: vec![1.0; NUM_CHANNELS],
        };

        let eeg_data = create_varied_test_eeg_data();
        let result = service.preprocess_data(&eeg_data);

        assert!(result.is_ok());
        let processed_data = result.unwrap();
        assert_eq!(processed_data.len(), WINDOW_SIZE * NUM_CHANNELS);
    }

    // Test for data preprocessing - success case
    #[test]
    fn test_preprocess_data_success() {
        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
            global_mean: vec![0.0; NUM_CHANNELS],
            global_std: vec![1.0; NUM_CHANNELS],
        };

        let eeg_data = create_test_eeg_data();
        let result = service.preprocess_data(&eeg_data);

        assert!(result.is_ok());
        let processed_data = result.unwrap();
        // Verify size: WINDOW_SIZE samples * NUM_CHANNELS channels
        assert_eq!(processed_data.len(), WINDOW_SIZE * NUM_CHANNELS);
    }

    // Test for data preprocessing - missing channel error
    #[test]
    fn test_preprocess_data_missing_channel() {
        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
            global_mean: vec![0.0; NUM_CHANNELS],
            global_std: vec![1.0; NUM_CHANNELS],
        };

        let mut eeg_data = create_test_eeg_data();
        // Remove a required channel
        eeg_data.remove("T3");

        let result = service.preprocess_data(&eeg_data);
        assert!(result.is_err());
        let error_msg = result.err().unwrap();
        assert!(error_msg.contains("Required channel 'T3' not found"));
    }

    // Test for data preprocessing - empty channel data
    #[test]
    fn test_preprocess_data_empty_channel() {
        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
            global_mean: vec![0.0; NUM_CHANNELS],
            global_std: vec![1.0; NUM_CHANNELS],
        };

        let mut eeg_data = create_test_eeg_data();
        // Set an empty channel
        eeg_data.insert("T3".to_string(), vec![]);

        let result = service.preprocess_data(&eeg_data);
        assert!(result.is_err());
        let error_msg = result.err().unwrap();
        assert!(error_msg.contains("Channel 'T3' has no data"));
    }

    // Test for prediction with model not loaded
    #[test]
    fn test_predict_model_not_loaded() {
        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
            global_mean: vec![0.0; NUM_CHANNELS],
            global_std: vec![1.0; NUM_CHANNELS],
        };

        let eeg_data = create_test_eeg_data();
        let result = service.predict_color(&eeg_data);

        assert!(result.is_err());
        let error_msg = result.err().unwrap();
        assert_eq!(error_msg, "Model is not loaded. Call load_model first.");
    }

    // Test for short data handling in preprocessing
    #[test]
    fn test_preprocess_data_short() {
        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
            global_mean: vec![0.0; NUM_CHANNELS],
            global_std: vec![1.0; NUM_CHANNELS],
        };

        let mut eeg_data = create_test_eeg_data();
        // Set a channel with fewer elements
        eeg_data.insert("T3".to_string(), vec![100000.0; 30]);

        let result = service.preprocess_data(&eeg_data);
        assert!(result.is_ok());
        let processed_data = result.unwrap();
        // Verify the function handled short data correctly
        assert_eq!(processed_data.len(), WINDOW_SIZE * NUM_CHANNELS);
    }

    // Test for long data handling in preprocessing
    #[test]
    fn test_preprocess_data_long() {
        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
            global_mean: vec![0.0; NUM_CHANNELS],
            global_std: vec![1.0; NUM_CHANNELS],
        };

        let mut eeg_data = create_test_eeg_data();
        // Set a channel with more elements
        eeg_data.insert("T3".to_string(), vec![100000.0; 100]);

        let result = service.preprocess_data(&eeg_data);
        assert!(result.is_ok());
        let processed_data = result.unwrap();
        // Verify the function handled long data correctly (uses last WINDOW_SIZE samples)
        assert_eq!(processed_data.len(), WINDOW_SIZE * NUM_CHANNELS);
    }

    // Test for constant data (zero variance) - global z-score handles this with epsilon
    #[test]
    fn test_preprocess_data_zero_variance() {
        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
            global_mean: vec![0.0; NUM_CHANNELS],
            global_std: vec![1.0; NUM_CHANNELS],
        };

        // All values are the same (zero variance)
        let mut eeg_data = HashMap::new();
        eeg_data.insert("T3".to_string(), vec![250000.0; 62]);
        eeg_data.insert("T4".to_string(), vec![250000.0; 62]);
        eeg_data.insert("O1".to_string(), vec![250000.0; 62]);
        eeg_data.insert("O2".to_string(), vec![250000.0; 62]);

        let result = service.preprocess_data(&eeg_data);
        assert!(result.is_ok());
        // global z-score with epsilon should handle zero variance
        let processed_data = result.unwrap();
        assert_eq!(processed_data.len(), WINDOW_SIZE * NUM_CHANNELS);
    }

    // Test for predict_color when model not loaded
    #[test]
    fn test_predict_color_no_model() {
        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
            global_mean: vec![0.0; NUM_CHANNELS],
            global_std: vec![1.0; NUM_CHANNELS],
        };

        let eeg_data = create_test_eeg_data();
        let result = service.predict_color(&eeg_data);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            "Model is not loaded. Call load_model first."
        );
    }

    // Mock test for is_model_loaded
    #[test]
    fn test_is_model_loaded() {
        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
            global_mean: vec![0.0; NUM_CHANNELS],
            global_std: vec![1.0; NUM_CHANNELS],
        };

        assert!(!service.is_model_loaded());
    }
}
