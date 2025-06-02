use crate::domain::{
    commands::search_headband_command::SearchHeadbandCommand, context::NeuralAnalyticsContext,
};
use log::{debug, error, info};
use presage::{command_handler, Error, Events};


/// This use case is responsible for searching and connecting to the EEG headset (BrainBit device).
/// It checks if the device is already connected and attempts to connect it.
/// If successful, it returns an empty list of events.
///
/// # Arguments
/// * `_context`: A mutable reference to the `NeuralAnalyticsContext` which contains
/// the EEG headset adapter.
/// * `_command`: The command to search and connect the headband.
///
/// # Returns
/// * `Result<Events, Error>`: A result containing either the events generated from
/// the connection or an error if something goes wrong.
#[command_handler(error = Error)]
pub async fn search_headband_use_case(
    _context: &mut NeuralAnalyticsContext,
    _command: SearchHeadbandCommand,
) -> Result<Events, Error> {
    info!("Starting search and connection of BrainBit device...");

    // Get the EEG headset adapter from the context
    let headset = _context.eeg_headset_adapter.read().await;

    // Check if already connected
    if headset.is_connected() {
        debug!("The device is already connected.");
        return Ok(Events::new());
    }

    // Try to connect to the device
    match headset.connect() {
        Ok(_) => {
            debug!("Connection established successfully.");
        }
        Err(e) => {
            let error_msg = format!("Error connecting to the device: {}", e);
            error!("{}", error_msg);
            return Err(Error::MissingCommandHandler(Box::leak(error_msg.into_boxed_str())).into());
        }
    }

    if headset.is_connected() {
        debug!("The device is now connected.");

        // Return an empty list of events for now
        Ok(Events::new())
    } else {
        let error_msg = "Error: Device is not connected or is not sending data. Connect first.";
        error!("{}", error_msg);
        return Err(Error::MissingCommandHandler(error_msg).into());
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
            fn get_work_mode(&self) -> crate::domain::models::eeg_work_modes::WorkMode;
            fn change_work_mode(&mut self, mode: crate::domain::models::eeg_work_modes::WorkMode);
            fn extract_impedance_data(&self) -> Result<std::collections::HashMap<String, u16>, String>;
            fn extract_raw_data(&self) -> Result<std::collections::HashMap<String, Vec<f32>>, String>;
        }
    }

    /// Función auxiliar para crear mocks estáticos para los tests
    /// Esta función crea un mock y lo convierte en una referencia estática
    /// que puede ser utilizada en el contexto del test.
    fn create_static_mock<T>(mock: T) -> &'static Arc<RwLock<Box<dyn EegHeadsetPort + Send + Sync>>>
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
        CommandBus::<NeuralAnalyticsContext, Error>::new()
            .configure(Configuration::new().command_handler(&search_headband_use_case))
    }

    #[test]
    async fn test_search_already_connected() {
        // Arrange
        let mut mock = MockEegHeadsetAdapter::new();
        mock.expect_is_connected().return_const(true); // Device is already connected

        let mut context = NeuralAnalyticsContext::default();
        context.eeg_headset_adapter = create_static_mock(mock);

        let command = SearchHeadbandCommand;
        let command_bus = setup_command_bus();

        // Act
        let result = command_bus.execute(&mut context, command).await;

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    async fn test_search_connect_successful() {
        // Arrange
        let mut mock = MockEegHeadsetAdapter::new();

        // Setup sequence of is_connected calls
        mock.expect_is_connected().times(1).returning(|| false); // Initially not connected

        mock.expect_connect().times(1).returning(|| Ok(()));

        mock.expect_is_connected().times(1).returning(|| true); // Connected after connect()

        let mut context = NeuralAnalyticsContext::default();
        context.eeg_headset_adapter = create_static_mock(mock);

        let command = SearchHeadbandCommand;
        let command_bus = setup_command_bus();

        // Act
        let result = command_bus.execute(&mut context, command).await;

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    async fn test_search_connect_error() {
        // Arrange
        let mut mock = MockEegHeadsetAdapter::new();

        mock.expect_is_connected().return_const(false); // Device is not connected

        mock.expect_connect()
            .times(1)
            .returning(|| Err("Failed to connect to device".to_string()));

        let mut context = NeuralAnalyticsContext::default();
        context.eeg_headset_adapter = create_static_mock(mock);

        let command = SearchHeadbandCommand;
        let command_bus = setup_command_bus();

        // Act
        let result = command_bus.execute(&mut context, command).await;

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Error connecting to the device"));
    }

    #[test]
    async fn test_search_connect_not_connected_after_attempt() {
        // Arrange
        let mut mock = MockEegHeadsetAdapter::new();

        mock.expect_is_connected().times(2).returning(|| false); // Never connected, even after connect()

        mock.expect_connect().times(1).returning(|| Ok(())); // Connect succeeds but device doesn't actually connect

        let mut context = NeuralAnalyticsContext::default();
        context.eeg_headset_adapter = create_static_mock(mock);

        let command = SearchHeadbandCommand;
        let command_bus = setup_command_bus();

        // Act
        let result = command_bus.execute(&mut context, command).await;

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Device is not connected or is not sending data"));
    }
}
