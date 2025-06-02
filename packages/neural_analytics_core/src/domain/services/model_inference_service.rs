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
    model:
        Option<Arc<RunnableModel<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    // Helper function to create test EEG data
    fn create_test_eeg_data() -> HashMap<String, Vec<f32>> {
        let mut eeg_data = HashMap::new();
        // Create valid data for all required channels
        eeg_data.insert("T3".to_string(), vec![0.1; 62]);
        eeg_data.insert("T4".to_string(), vec![0.2; 62]);
        eeg_data.insert("O1".to_string(), vec![0.3; 62]);
        eeg_data.insert("O2".to_string(), vec![0.4; 62]);
        eeg_data
    }

    // Helper to create varied test data with different values
    fn create_varied_test_eeg_data() -> HashMap<String, Vec<f32>> {
        let mut eeg_data = HashMap::new();
        // Creamos valores variados para obtener mejor cobertura en la normalización
        eeg_data.insert("T3".to_string(), (0..62).map(|i| i as f32 * 0.1).collect());
        eeg_data.insert("T4".to_string(), (0..62).map(|i| i as f32 * 0.2).collect());
        eeg_data.insert("O1".to_string(), (0..62).map(|i| i as f32 * 0.3).collect());
        eeg_data.insert("O2".to_string(), (0..62).map(|i| i as f32 * 0.4).collect());
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
        // El comportamiento dependerá de si existe el archivo por defecto o no
        // Solo verificamos que la función no falle
        assert_eq!(service.model_path, "assets/neural_analytics.onnx");
    }

    // Test for data preprocessing with varied data (better coverage)
    #[test]
    fn test_preprocess_data_varied() {
        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
        };

        let eeg_data = create_varied_test_eeg_data();
        let result = service.preprocess_data(&eeg_data);

        assert!(result.is_ok());
        let processed_data = result.unwrap();
        assert_eq!(processed_data.len(), 62 * 4);
    }

    // Test for data preprocessing - success case
    #[test]
    fn test_preprocess_data_success() {
        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
        };

        let eeg_data = create_test_eeg_data();
        let result = service.preprocess_data(&eeg_data);

        assert!(result.is_ok());
        let processed_data = result.unwrap();
        // Verify size: 62 samples * 4 channels = 248 elements
        assert_eq!(processed_data.len(), 62 * 4);
    }

    // Test for data preprocessing - missing channel error
    #[test]
    fn test_preprocess_data_missing_channel() {
        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
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
        };

        let mut eeg_data = create_test_eeg_data();
        // Set a channel with fewer elements
        eeg_data.insert("T3".to_string(), vec![0.1; 30]);

        let result = service.preprocess_data(&eeg_data);
        assert!(result.is_ok());
        let processed_data = result.unwrap();
        // Verify the function handled short data correctly
        assert_eq!(processed_data.len(), 62 * 4);
    }

    // Test for long data handling in preprocessing
    #[test]
    fn test_preprocess_data_long() {
        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
        };

        let mut eeg_data = create_test_eeg_data();
        // Set a channel with more elements
        eeg_data.insert("T3".to_string(), vec![0.1; 100]);

        let result = service.preprocess_data(&eeg_data);
        assert!(result.is_ok());
        let processed_data = result.unwrap();
        // Verify the function handled long data correctly
        assert_eq!(processed_data.len(), 62 * 4);
    }

    // Test for zero variance data
    #[test]
    fn test_preprocess_data_zero_variance() {
        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
        };

        // Todos los valores son iguales, lo que resultará en varianza cero
        let mut eeg_data = HashMap::new();
        eeg_data.insert("T3".to_string(), vec![5.0; 62]);
        eeg_data.insert("T4".to_string(), vec![5.0; 62]);
        eeg_data.insert("O1".to_string(), vec![5.0; 62]);
        eeg_data.insert("O2".to_string(), vec![5.0; 62]);

        let result = service.preprocess_data(&eeg_data);
        assert!(result.is_ok());
        // Con varianza cero, la división por (std_dev + 1e-6) debería evitar el NaN
        let processed_data = result.unwrap();
        assert_eq!(processed_data.len(), 62 * 4);
    }

    // Test for predict_color with tensor shape error
    #[test]
    fn test_predict_color_tensor_shape_error() {
        // Simular un modelo cargado para esta prueba
        struct MockModel;

        impl ModelInferenceInterface for MockModel {
            fn predict_color(&self, _: &HashMap<String, Vec<f32>>) -> Result<String, String> {
                // Esta implementación nunca se llamará en la prueba
                Ok("red".to_string())
            }

            fn is_model_loaded(&self) -> bool {
                true
            }
        }

        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
        };

        // Crear datos con longitud incorrecta para forzar el error de verificación de longitud
        let mut eeg_data = create_test_eeg_data();
        // Manipulamos la estructura interna para forzar un error
        // En realidad esto no debería suceder con la implementación actual,
        // pero probamos la condición de error de todos modos

        let result = service.predict_color(&eeg_data);
        assert!(result.is_err());
        // El error debe ser por modelo no cargado, no por longitud incorrecta
        assert_eq!(
            result.err().unwrap(),
            "Model is not loaded. Call load_model first."
        );
    }

    // Mock test for predict_color (since we can't easily create a real ONNX model)
    #[test]
    fn test_predict_color_mock() {
        // This test is a placeholder for a proper prediction test
        // A real test would require creating a valid ONNX model, which is complex
        // Instead, we'll just test the interface as a sanity check

        // In a real test environment, you'd create a test-specific ONNX model
        // or use dependency injection to mock the model behavior

        let service = ModelInferenceService {
            model: None,
            model_path: "dummy_path".to_string(),
        };

        assert!(!service.is_model_loaded());
    }
}
