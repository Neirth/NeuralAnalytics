use presage::{command_handler, Error, Events};
use crate::domain::{
    commands::extract_calibration_data_command::ExtractCalibrationDataCommand, 
    context::NeuralAnalyticsContext, 
    models::{eeg_work_modes::WorkMode, event_internals::ReceivedCalibrationDataEvent}, 
    ports::input::eeg_headset::EegHeadsetPort
};
use std::collections::HashMap;
use log::{self, info};

/// This use case is responsible for extracting calibration data from the EEG headset
/// and processing it. It checks if the device is connected and in the correct mode
/// before attempting to extract the data. The extracted data is then processed and
/// returned as an event.
/// 
/// # Arguments
/// * `_context`: A mutable reference to the `NeuralAnalyticsContext` which contains
///  the EEG headset adapter.
/// * `_command`: The command to extract calibration data.
///
/// # Returns
/// * `Result<Events, Error>`: A result containing either the events generated from
///  the extracted data or an error if something goes wrong.
#[command_handler(error = Error)]
pub async fn extract_calibration_data_use_case(
    _context: &mut NeuralAnalyticsContext,
    _command: ExtractCalibrationDataCommand,
) -> Result<Events, Error> {
    log::info!("Starting calibration data extraction from BrainBit device...");

    // Obtain the EEG headset adapter from the context
    let mut headset_guard = _context.eeg_headset_adapter.write().await;
    let headset: &mut dyn EegHeadsetPort = headset_guard.as_mut();

    // Check if the device is connected
    if !headset.is_connected() {
        let error_msg = "Error: Device is not connected. Connect first.";
        log::error!("{}", error_msg);
        return Err(Error::MissingCommandHandler(error_msg).into());
    }

    if headset.get_work_mode() != WorkMode::Calibration {
        log::info!("Changing work mode to Calibration...");
        headset.change_work_mode(WorkMode::Calibration);
    }
    
    let data = match headset.extract_impedance_data() {
        Ok(data) => {
            process_impedance_data(&data);
            log::info!("Calibration data successfully extracted.");
            data
        },
        Err(e) => {
            let error_msg = format!("Error extracting calibration data from device: {}", e);
            log::error!("{}", error_msg);
            return Err(Error::MissingCommandHandler(Box::leak(error_msg.into_boxed_str())).into());
        }
    };

    let mut events = Events::new();

    let _ = events.add(ReceivedCalibrationDataEvent {
        impedance_data: data,
    });

    Ok(events)
}

// Helper function to process impedance data
fn process_impedance_data(data: &HashMap<String, u16>) {
    info!("Processing electrode impedance data:");
    for (electrode, last_value) in data {            
        let status = if *last_value > 2 {
            "ERROR: Poor connection"
        } else if *last_value >= 1 && *last_value <= 2 {
            "WARNING: Acceptable connection"
        } else {
            "OK: Good connection"
        };
        
        info!("  Electrode {}: {:.2} kOhm - {}", electrode, last_value, status);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::domain::ports::input::eeg_headset::EegHeadsetPort;
    use mockall::predicate::*;
    use mockall::mock;
    use presage::CommandBus;
    use presage::Configuration;
    use tokio::sync::RwLock;
    use tokio::test;

    // Mock implementation of the EegHeadsetPort for testing
    mock! {
        EegHeadsetAdapter {}
        impl EegHeadsetPort for EegHeadsetAdapter {
            fn connect(&self) -> Result<(), String>;
            fn disconnect(&mut self) -> Result<(), String>;
            fn is_connected(&self) -> bool;
            fn get_work_mode(&self) -> WorkMode;
            fn change_work_mode(&mut self, mode: WorkMode);
            fn extract_impedance_data(&self) -> Result<HashMap<String, u16>, String>;
            fn extract_raw_data(&self) -> Result<HashMap<String, Vec<f32>>, String>;
        }
    }

    /// Función auxiliar para crear mocks estáticos para los tests
    /// Esta función crea un mock y lo convierte en una referencia estática
    /// que puede ser utilizada en el contexto del test.
    fn create_static_mock<T>(
        mock: T,
    ) -> &'static Arc<RwLock<Box<dyn EegHeadsetPort + Send + Sync>>>
    where
        T: EegHeadsetPort + Send + Sync + 'static,
    {
        // Crear un Box dinámico con el mock
        let boxed_mock: Box<dyn EegHeadsetPort + Send + Sync> = Box::new(mock);

        // Envolver en RwLock y Arc
        let arc_rwlock = Arc::new(RwLock::new(boxed_mock));

        // Convertir a referencia estática
        Box::leak(Box::new(arc_rwlock))
    }

    /// Función auxiliar para configurar el CommandBus para los tests
    fn setup_command_bus() -> CommandBus<NeuralAnalyticsContext, Error> {
        CommandBus::<NeuralAnalyticsContext, Error>::new().configure(
            Configuration::new()
                .command_handler(&extract_calibration_data_use_case)
        )
    }

    #[test]
    async fn test_extract_calibration_data_disconnected() {
        // Arrange
        let mut mock = MockEegHeadsetAdapter::new();
        mock.expect_is_connected()
            .return_const(false); // Device is not connected

        let mut context = NeuralAnalyticsContext::default();
        context.eeg_headset_adapter = create_static_mock(mock);

        let command = ExtractCalibrationDataCommand;
        let command_bus = setup_command_bus();

        // Act
        let result = command_bus.execute(&mut context, command).await;

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Device is not connected"));
    }

    #[test]
    async fn test_extract_calibration_data_already_in_calibration_mode() {
        // Arrange
        let mut mock = MockEegHeadsetAdapter::new();
        mock.expect_is_connected()
            .return_const(true); // Device is connected
            
        mock.expect_get_work_mode()
            .return_const(WorkMode::Calibration); // Already in calibration mode
            
        // We don't expect change_work_mode to be called
        
        let mut impedance_data = HashMap::new();
        impedance_data.insert("electrode1".to_string(), 1);
        impedance_data.insert("electrode2".to_string(), 2);
        
        mock.expect_extract_impedance_data()
            .times(1)
            .returning(move || Ok(impedance_data.clone()));

        let mut context = NeuralAnalyticsContext::default();
        context.eeg_headset_adapter = create_static_mock(mock);

        let command = ExtractCalibrationDataCommand;
        let command_bus = setup_command_bus();

        // Act
        let result = command_bus.execute(&mut context, command).await;

        // Assert
        assert!(result.is_ok());
        
        // Verify that the context has been updated with the impedance data
        // Since this is using the CommandBus, we can't directly access the events
        // Instead, we can check if the context was properly updated
        // If needed, we could implement a custom handler to capture and verify the events
    }

    #[test]
    async fn test_extract_calibration_data_change_mode() {
        // Arrange
        let mut mock = MockEegHeadsetAdapter::new();
        mock.expect_is_connected()
            .return_const(true); // Device is connected
            
        mock.expect_get_work_mode()
            .return_const(WorkMode::Extraction); // In extraction mode
            
        mock.expect_change_work_mode()
            .times(1)
            .with(eq(WorkMode::Calibration))
            .return_const(());
            
        let mut impedance_data = HashMap::new();
        impedance_data.insert("electrode1".to_string(), 1);
        
        mock.expect_extract_impedance_data()
            .times(1)
            .returning(move || Ok(impedance_data.clone()));

        let mut context = NeuralAnalyticsContext::default();
        context.eeg_headset_adapter = create_static_mock(mock);

        let command = ExtractCalibrationDataCommand;
        let command_bus = setup_command_bus();

        // Act
        let result = command_bus.execute(&mut context, command).await;

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    async fn test_extract_calibration_data_extraction_error() {
        // Arrange
        let mut mock = MockEegHeadsetAdapter::new();
        mock.expect_is_connected()
            .return_const(true); // Device is connected
            
        mock.expect_get_work_mode()
            .return_const(WorkMode::Calibration); // Already in calibration mode
            
        mock.expect_extract_impedance_data()
            .times(1)
            .returning(|| Err("Impedance extraction failed".to_string()));

        let mut context = NeuralAnalyticsContext::default();
        context.eeg_headset_adapter = create_static_mock(mock);

        let command = ExtractCalibrationDataCommand;
        let command_bus = setup_command_bus();

        // Act
        let result = command_bus.execute(&mut context, command).await;

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Error extracting calibration data"));
    }
}