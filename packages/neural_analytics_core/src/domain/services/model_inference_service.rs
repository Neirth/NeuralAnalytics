use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;
use log::{info, error, warn};
use tract_onnx::prelude::*;

// Trait that defines the interface for the inference service
pub trait ModelInferenceInterface: Send + Sync + 'static {
    /// Predicts the color the user is thinking based on EEG data
    fn predict_color(&self, eeg_data: &HashMap<String, Vec<f32>>) -> Result<String, String>;
    
    /// Checks if the model is loaded and ready for predictions
    fn is_model_loaded(&self) -> bool;
}

/// Service to perform inferences with a pre-trained ONNX model
/// for color prediction based on EEG data, using Tract
pub struct ModelInferenceService {
	// Changed to concrete SimplePlan type, as model.into_runnable() returns this type.
	// Previously: model: Option<Arc<dyn TypedOp + Send + Sync>>,
	model: Option<Arc<SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>>,
    // The model will be loaded as an optimized graph
    model_path: String,
}

impl Default for ModelInferenceService {
    fn default() -> Self {
        // Define the default path to the model
        let model_path = "assets/model.onnx".to_string();
        let mut service = Self {
            model: None,
            model_path,
        };
        
        // Try to load the model automatically
        match service.load_model() {
            Ok(_) => info!("ONNX model successfully loaded with Tract"),
            Err(e) => warn!("Could not load the model automatically: {}", e),
        }
        
        service
    }
}

impl Drop for ModelInferenceService {
    fn drop(&mut self) {
        // Explicitly release resources if necessary
        if self.model.is_some() {
            info!("Releasing Tract model resources");
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
    
    /// Loads the ONNX model from the specified path using Tract
    fn load_model(&mut self) -> Result<(), String> {
        let path = Path::new(&self.model_path);
        
        if !path.exists() {
            return Err(format!("Model file does not exist at path: {}", self.model_path));
        }
        
        // Load and optimize model with Tract
        let model = tract_onnx::onnx()
            // Load the model from file
            .model_for_path(&self.model_path)
            .map_err(|e| format!("Error loading the model: {}", e))
            // Optimize for the expected input
            .and_then(|mut model| {
                // Replace reshape_inputs with with_input_fact to define the input shape
                model.with_input_fact(0, InferenceFact::dt_shape(f32::datum_type(), tvec!(1, -1)))
                    .map_err(|e| format!("Error configuring input shape: {}", e))
            })
            // Optimize the model and convert it to an inference version
            .and_then(|model| {
                model.into_optimized()
                    .map_err(|e| format!("Error optimizing the model: {}", e))
            })
            // Finalize the model build
            .and_then(|model| {
                model.into_runnable()
                    .map_err(|e| format!("Error finalizing the model: {}", e))
            })?;
            
        // Save the concrete model in our service
        self.model = Some(Arc::new(model));
        
        Ok(())
    }
    
    /// Preprocesses the EEG data before passing it to the model
    /// This function implements the same preprocessing used in training
    fn preprocess_data(&self, eeg_data: &HashMap<String, Vec<f32>>) -> Result<Vec<f32>, String> {
        // Check that the required channels are present
        let required_channels = ["T3", "T4", "O1", "O2"];
        for channel in required_channels.iter() {
            if !eeg_data.contains_key(*channel) {
                return Err(format!("Required channel '{}' not found in EEG data", channel));
            }
        }
        
        // Extract and concatenate the channel data into a one-dimensional vector
        // following the order expected by the trained model
        let mut processed_data = Vec::new();
        
        for channel in required_channels.iter() {
            let channel_data = eeg_data.get(*channel).unwrap();
            
            // Apply normalization similar to that used in training
            // We assume that we already have filtered data (e.g., alpha band 8-13Hz)
            let mean = channel_data.iter().sum::<f32>() / channel_data.len() as f32;
            let variance = channel_data.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f32>() / channel_data.len() as f32;
            let std_dev = variance.sqrt();
            
            // Normalize and add the channel data to the input vector
            let normalized = channel_data.iter()
                .map(|&x| (x - mean) / (std_dev + 1e-6))
                .collect::<Vec<f32>>();
            
            processed_data.extend(normalized);
        }
        
        Ok(processed_data)
    }
}

impl ModelInferenceInterface for ModelInferenceService {
    fn predict_color(&self, eeg_data: &HashMap<String, Vec<f32>>) -> Result<String, String> {
        // Check that the model is loaded
        let model = match &self.model {
            Some(model) => model,
            None => return Err("Model is not loaded. Call load_model first.".to_string()),
        };
        
        // Preprocess the data
        let processed_data = self.preprocess_data(eeg_data)?;
        
        // Convert processed data to a Tract tensor
        let input_tensor = match tract_ndarray::Array::from_shape_vec(
            [1, processed_data.len()],
            processed_data
        ) {
            Ok(tensor) => tensor,
            Err(e) => return Err(format!("Error creating input tensor: {}", e))
        };
        
        // Perform inference
        let result = match model.run(tvec!(input_tensor.into_tensor().into())) {
            Ok(outputs) => outputs,
            Err(e) => return Err(format!("Error during inference: {}", e)),
        };
        
        // Get the output tensor (probabilities) and convert to Vec
        let output_tensor = match &result[0].as_slice::<f32>() {
            Ok(slice) => slice.to_vec(),
            Err(e) => return Err(format!("Error getting model probabilities: {}", e)),
        };
        
        // Map indices to colors (adjust according to model classes)
        let color_map = ["red", "green", "blue", "yellow", "purple"];
        
        if output_tensor.is_empty() {
            return Err("No probabilities obtained from the model".to_string());
        }
        
        // Find the color with the highest probability
        let mut max_prob = output_tensor[0];
        let mut max_idx = 0;
        
        for (i, &prob) in output_tensor.iter().enumerate() {
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
