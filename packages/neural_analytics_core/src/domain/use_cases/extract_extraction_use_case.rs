use crate::domain::{
    commands::extract_generalist_data_command::ExtractGeneralistDataCommand,
    context::NeuralAnalyticsContext,
    models::{eeg_work_modes::WorkMode, event_internals::ReceivedGeneralistDataEvent},
    ports::input::eeg_headset::EegHeadsetPort,
};
use log::{error, info};
use presage::{command_handler, Error, Events};
use std::collections::HashMap;

/// This use case is responsible for extracting raw EEG data from the EEG headset
/// and processing it. It checks if the device is connected and in the correct mode
/// before attempting to extract the data. The extracted data is then processed and
/// returned as an event.
///
/// # Arguments
/// * `_context`: A mutable reference to the `NeuralAnalyticsContext` which contains
/// the EEG headset adapter.
/// * `_command`: The command to extract generalist data.
///
/// # Returns
/// * `Result<Events, Error>`: A result containing either the events generated from
/// the extracted data or an error if something goes wrong.
#[command_handler(error = Error)]
pub async fn extract_generalist_data_use_case(
    _context: &mut NeuralAnalyticsContext,
    _command: ExtractGeneralistDataCommand,
) -> Result<Events, Error> {
    info!("Starting raw data extraction from BrainBit device...");

    // Get the EEG headset adapter from the context
    let mut headset_guard = _context.eeg_headset_adapter.write().await;
    let headset: &mut dyn EegHeadsetPort = headset_guard.as_mut();

    // Check if the device is connected
    if !headset.is_connected() {
        let error_msg = "Error: Device is not connected. Connect first.";
        error!("{}", error_msg);
        return Err(Error::MissingCommandHandler(error_msg).into());
    }

    // Change to extraction mode before trying to get data
    if headset.get_work_mode() != WorkMode::Extraction {
        info!("Changing work mode to Extraction...");
        headset.change_work_mode(WorkMode::Extraction);
    }

    // Try to extract raw data from the device
    let data = match headset.extract_raw_data() {
        Ok(data) => {
            // Process the extracted data
            process_eeg_data(&data);
            data
        }
        Err(e) => {
            let error_msg = format!("Error extracting data from device: {}", e);
            error!("{}", error_msg);
            return Err(Error::MissingCommandHandler(Box::leak(error_msg.into_boxed_str())).into());
        }
    };

    // Create event with the extracted data
    let mut events = Events::new();
    let _ = events.add(ReceivedGeneralistDataEvent { headset_data: data });

    // Send the event to the event queue
    Ok(events)
}

// Helper function to process the EEG data
fn process_eeg_data(data: &HashMap<String, Vec<f32>>) {
    // For now, we simply show basic information about the received data
    info!("Processing EEG data:");
    for (channel, values) in data {
        info!("  Channel {}: {} samples received", channel, values.len());
        if !values.is_empty() {
            info!(
                "    - First values: {:?}",
                &values[..std::cmp::min(values.len(), 5)]
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::domain::ports::input::eeg_headset::EegHeadsetPort;
    use mockall::mock;
    use mockall::predicate::*;
    use presage::CommandBus;
    use tokio::sync::RwLock;
    use tokio::test;
    use presage::Configuration;

    // Mock implementation of the EegHeadsetPort for testing
    mock! {
        EegHeadsetAdapter {}
        impl EegHeadsetPort for EegHeadsetAdapter {
            fn connect(&self) -> Result<(), String>;
            fn disconnect(&mut self) -> Result<(), String>; // Corregido de &self a &mut self
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
                .command_handler(&extract_generalist_data_use_case)
        )
    }

    #[test]
    async fn test_extract_generalist_data_disconnected() {
        // Arrange
        let mut mock = MockEegHeadsetAdapter::new();
        mock.expect_is_connected().return_const(false); // Device is not connected

        let mut context = NeuralAnalyticsContext::default();
        context.eeg_headset_adapter = create_static_mock(mock);

        let command = ExtractGeneralistDataCommand;
        let command_bus = setup_command_bus();

        // Act
        let result = command_bus.execute(&mut context, command).await;

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Device is not connected"));
    }

    #[test]
    async fn test_extract_generalist_data_already_in_extraction_mode() {
        // Arrange
        let mut mock = MockEegHeadsetAdapter::new();
        mock.expect_is_connected().return_const(true); // Device is connected

        mock.expect_get_work_mode()
            .return_const(WorkMode::Extraction); // Already in extraction mode

        // We don't expect change_work_mode to be called

        let mut eeg_data = HashMap::new();
        eeg_data.insert("channel1".to_string(), vec![1.0, 2.0, 3.0]);
        eeg_data.insert("channel2".to_string(), vec![4.0, 5.0, 6.0]);

        mock.expect_extract_raw_data()
            .times(1)
            .returning(move || Ok(eeg_data.clone()));

        let mut context = NeuralAnalyticsContext::default();
        context.eeg_headset_adapter = create_static_mock(mock);

        let command = ExtractGeneralistDataCommand;
        let command_bus = setup_command_bus();

        // Act
        let result = command_bus.execute(&mut context, command).await;

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    async fn test_extract_generalist_data_change_mode() {
        // Arrange
        let mut mock = MockEegHeadsetAdapter::new();
        mock.expect_is_connected().return_const(true); // Device is connected

        mock.expect_get_work_mode()
            .return_const(WorkMode::Calibration); // In calibration mode

        mock.expect_change_work_mode()
            .times(1)
            .with(eq(WorkMode::Extraction))
            .return_const(());

        let mut eeg_data = HashMap::new();
        eeg_data.insert("channel1".to_string(), vec![1.0, 2.0, 3.0]);

        mock.expect_extract_raw_data()
            .times(1)
            .returning(move || Ok(eeg_data.clone()));

        let mut context = NeuralAnalyticsContext::default();
        context.eeg_headset_adapter = create_static_mock(mock);

        let command = ExtractGeneralistDataCommand;
        let command_bus = setup_command_bus();

        // Act
        let result = command_bus.execute(&mut context, command).await;

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    async fn test_extract_generalist_data_extraction_error() {
        // Arrange
        let mut mock = MockEegHeadsetAdapter::new();
        mock.expect_is_connected().return_const(true); // Device is connected

        mock.expect_get_work_mode()
            .return_const(WorkMode::Extraction); // Already in extraction mode

        mock.expect_extract_raw_data()
            .times(1)
            .returning(|| Err("Raw data extraction failed".to_string()));

        let mut context = NeuralAnalyticsContext::default();
        context.eeg_headset_adapter = create_static_mock(mock);

        let command = ExtractGeneralistDataCommand;
        let command_bus = setup_command_bus();

        // Act
        let result = command_bus.execute(&mut context, command).await;

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Error extracting data from device"));
    }
}
