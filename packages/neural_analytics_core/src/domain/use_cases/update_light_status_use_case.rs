use crate::domain::{
    commands::update_light_status_command::UpdateLightStatusCommand,
    context::NeuralAnalyticsContext, models::bulb_state::BulbState,
};
use log::info;
use presage::{command_handler, Error, Events};


/// This use case is responsible for updating the light status of a smart bulb.
/// It checks if the command is valid and then sends the appropriate command
/// to the smart bulb adapter to change its state.
///
/// # Arguments
/// * `_context`: A mutable reference to the `NeuralAnalyticsContext` which contains
/// the smart bulb adapter.
/// * `command`: The command to update the light status.
///
/// # Returns
/// * `Result<Events, Error>`: A result containing either the events generated from
/// the update or an error if something goes wrong.
#[command_handler(error = Error)]
pub async fn update_light_status_use_case(
    _context: &mut NeuralAnalyticsContext,
    command: UpdateLightStatusCommand,
) -> Result<Events, Error> {
    // Parse the command to extract the desired light status
    match command.is_light_on {
        true => {
            info!("Turning the light on...");

            // Obtain the smart bulb adapter from the context
            let smart_bulb = _context.smart_bulb_adapter.read().await;
            smart_bulb
                .change_state(BulbState::BulbOn)
                .await
                .map_err(|e| {
                    Error::MissingCommandHandler(Box::leak(e.to_string().into_boxed_str()))
                })?;
        }
        false => {
            info!("Turning the light off...");

            // Obtain the lock asynchronously for the change_state method
            let smart_bulb = _context.smart_bulb_adapter.read().await;
            smart_bulb
                .change_state(BulbState::BulbOff)
                .await
                .map_err(|e| {
                    Error::MissingCommandHandler(Box::leak(e.to_string().into_boxed_str()))
                })?;
        }
    }

    // Return an empty list of events for now
    Ok(Events::new())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::domain::ports::output::smart_bulb::SmartBulbPort;

    use super::*;
    use mockall::mock;
    use mockall::predicate::*;
    use presage::CommandBus;
    use presage::Configuration;
    use tokio::sync::RwLock;

    // Mock implementation of the SmartBulbPort for testing
    mock! {
        SmartBulbAdapter {}
        #[async_trait::async_trait]
        impl SmartBulbPort for SmartBulbAdapter {
            async fn change_state(&self, state: BulbState) -> Result<(), String>;
        }
    }

    /// Función auxiliar para crear mocks estáticos para los tests
    /// Esta función crea un mock y lo convierte en una referencia estática
    /// que puede ser utilizada en el contexto del test.
    fn create_static_mock<T>(mock: T) -> &'static Arc<RwLock<Box<dyn SmartBulbPort + Send + Sync>>>
    where
        T: SmartBulbPort + Send + Sync + 'static,
    {
        // Crear un Box dinámico con el mock
        let boxed_mock: Box<dyn SmartBulbPort + Send + Sync> = Box::new(mock);

        // Envolver en RwLock y Arc
        let arc_rwlock = Arc::new(RwLock::new(boxed_mock));

        // Convertir a referencia estática
        Box::leak(Box::new(arc_rwlock))
    }

    /// Función auxiliar para configurar el CommandBus para los tests
    fn setup_command_bus() -> CommandBus<NeuralAnalyticsContext, Error> {
        CommandBus::<NeuralAnalyticsContext, Error>::new()
            .configure(Configuration::new().command_handler(&update_light_status_use_case))
    }

    #[tokio::test]
    async fn test_update_light_status_turn_on_successful() {
        // Arrange
        let mut mock = MockSmartBulbAdapter::new();

        // Expect change_state to be called with BulbOn state
        mock.expect_change_state()
            .with(eq(BulbState::BulbOn))
            .times(1)
            .returning(|_| Ok(()));

        let mut context = NeuralAnalyticsContext::default();
        context.smart_bulb_adapter = create_static_mock(mock);

        let command = UpdateLightStatusCommand { is_light_on: true };
        let command_bus = setup_command_bus();

        // Act
        let result = command_bus.execute(&mut context, command).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_light_status_turn_off_successful() {
        // Arrange
        let mut mock = MockSmartBulbAdapter::new();

        // Expect change_state to be called with BulbOff state
        mock.expect_change_state()
            .with(eq(BulbState::BulbOff))
            .times(1)
            .returning(|_| Ok(()));

        let mut context = NeuralAnalyticsContext::default();
        context.smart_bulb_adapter = create_static_mock(mock);

        let command = UpdateLightStatusCommand { is_light_on: false };
        let command_bus = setup_command_bus();

        // Act
        let result = command_bus.execute(&mut context, command).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_light_status_turn_on_error() {
        // Arrange
        let mut mock = MockSmartBulbAdapter::new();

        // Expect change_state to be called and return an error
        mock.expect_change_state()
            .with(eq(BulbState::BulbOn))
            .times(1)
            .returning(|_| Err("Failed to turn on bulb".to_string()));

        let mut context = NeuralAnalyticsContext::default();
        context.smart_bulb_adapter = create_static_mock(mock);

        let command = UpdateLightStatusCommand { is_light_on: true };
        let command_bus = setup_command_bus();

        // Act
        let result = command_bus.execute(&mut context, command).await;

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to turn on bulb"));
    }

    #[tokio::test]
    async fn test_update_light_status_turn_off_error() {
        // Arrange
        let mut mock = MockSmartBulbAdapter::new();

        // Expect change_state to be called and return an error
        mock.expect_change_state()
            .with(eq(BulbState::BulbOff))
            .times(1)
            .returning(|_| Err("Failed to turn off bulb".to_string()));

        let mut context = NeuralAnalyticsContext::default();
        context.smart_bulb_adapter = create_static_mock(mock);

        let command = UpdateLightStatusCommand { is_light_on: false };
        let command_bus = setup_command_bus();

        // Act
        let result = command_bus.execute(&mut context, command).await;

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to turn off bulb"));
    }
}
