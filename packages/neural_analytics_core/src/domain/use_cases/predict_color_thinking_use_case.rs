use crate::domain::{
    commands::predict_color_thinking_command::PredictColorThinkingCommand,
    context::NeuralAnalyticsContext,
    models::event_internals::ReceivedPredictColorThinkingDataEvent,
};
use log::{error, info};
use presage::{command_handler, Error, Events};

/// Este caso de uso es responsable de predecir el color en el que está pensando el usuario
/// basado en los datos del EEG. Verifica si el auricular EEG está conectado y si los datos
/// están disponibles. Si los datos están disponibles, utiliza el servicio de modelo para predecir
/// el color y devuelve el resultado como un evento.
///
/// # Argumentos
/// * `_context`: Una referencia mutable al `NeuralAnalyticsContext` que contiene
/// el adaptador del auricular EEG y el servicio de modelo.
/// * `_command`: El comando para predecir el color en el que está pensando el usuario.
///
/// # Retorna
/// * `Result<Events, Error>`: Un resultado que contiene los eventos generados a partir de
/// la predicción o un error si algo sale mal.
#[command_handler(error = Error)]
pub async fn predict_color_thinking_use_case(
    _context: &mut NeuralAnalyticsContext,
    _command: PredictColorThinkingCommand,
) -> Result<Events, Error> {
    info!("Starting color prediction for what the user is thinking...");

    // Verificar si los datos del EEG están disponibles
    let headset_data = match &_context.headset_data {
        Some(data) => data,
        None => {
            let error_msg = "No EEG data available for prediction";
            error!("{}", error_msg);
            return Err(Error::MissingCommandHandler(error_msg).into());
        }
    };

    let model_service = _context.model_service.read().await;

    // Usar el servicio de inferencia para predecir el color
    info!("Processing EEG data for prediction...");
    let color_result = model_service.predict_color(headset_data).map_err(|e| {
        let error_msg = format!("Error predicting color: {}", e);
        error!("{}", error_msg);
        Error::MissingCommandHandler(Box::leak(error_msg.into_boxed_str()))
    })?;

    // Guardar el resultado en el contexto
    info!(
        "Successful prediction: the user is thinking of the color '{}'",
        color_result
    );

    // Crear y devolver eventos
    let mut events = Events::new();
    let _ = events.add(ReceivedPredictColorThinkingDataEvent {
        color_thinking: color_result,
    });

    // Enviar el evento a la cola de eventos
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::services::model_inference_service::ModelInferenceInterface as ModelServicePort;
    use mockall::mock;
    use mockall::predicate::*;
    use presage::{CommandBus, Configuration};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Implementación mock de ModelServicePort para las pruebas
    mock! {
        ModelService {}
        impl ModelServicePort for ModelService {
            fn predict_color(&self, headset_data: &HashMap<String, Vec<f32>>) -> Result<String, String>;
            fn is_model_loaded(&self) -> bool;
        }
    }

    /// Función auxiliar para crear mocks estáticos para los tests
    /// Esta función crea un mock y lo convierte en una referencia estática
    /// que puede ser utilizada en el contexto del test.
    fn create_static_mock<T>(
        mock: T,
    ) -> &'static Arc<RwLock<Box<dyn ModelServicePort + Send + Sync>>>
    where
        T: ModelServicePort + Send + Sync + 'static,
    {
        // Crear un Box dinámico con el mock
        let boxed_mock: Box<dyn ModelServicePort + Send + Sync> = Box::new(mock);

        // Envolver en RwLock y Arc
        let arc_rwlock = Arc::new(RwLock::new(boxed_mock));

        // Convertir a referencia estática
        Box::leak(Box::new(arc_rwlock))
    }

    /// Función auxiliar para configurar el CommandBus para los tests
    /// Ahora se requiere que el handler tenga lifetime 'static.
    fn setup_command_bus() -> CommandBus<NeuralAnalyticsContext, Error> {
        CommandBus::<NeuralAnalyticsContext, Error>::new().configure(
            Configuration::new()
                .command_handler(&predict_color_thinking_use_case)
            )
    }

    #[tokio::test]
    async fn test_predict_color_thinking_no_data() {
        // Arrange
        let mut context = NeuralAnalyticsContext::default();
        context.headset_data = None; // No hay datos disponibles

        // Asegurarnos de que model_service no es nulo
        let mock = MockModelService::new();
        context.model_service = create_static_mock(mock);

        let command = PredictColorThinkingCommand {};
        let command_bus = setup_command_bus();

        // Act
        let _ = command_bus.execute(&mut context, command).await;
        assert!(context.color_thinking.is_empty());
    }

    #[tokio::test]
    async fn test_predict_color_thinking_successful() {
        // Arrange
        let mut mock = MockModelService::new();

        let mut headset_data = HashMap::new();
        headset_data.insert("channel1".to_string(), vec![1.0, 2.0, 3.0]);

        mock.expect_predict_color()
            .times(1)
            .withf(move |data: &HashMap<String, Vec<f32>>| data.contains_key("channel1"))
            .returning(|_| Ok("green".to_string()));

        let mut context = NeuralAnalyticsContext::default();
        context.headset_data = Some(headset_data);
        context.model_service = create_static_mock(mock);

        let command = PredictColorThinkingCommand {};
        let command_bus = setup_command_bus();

        let _ = command_bus.execute(&mut context, command).await;

        assert!(!context.color_thinking.is_empty());
        assert_eq!(context.get_color_thinking(), "green".to_string());
    }

    #[tokio::test]
    async fn test_predict_color_thinking_prediction_error() {
        // Arrange
        let mut mock = MockModelService::new();

        let mut headset_data = HashMap::new();
        headset_data.insert("channel1".to_string(), vec![1.0, 2.0, 3.0]);

        mock.expect_predict_color()
            .times(1)
            .returning(|_| Err("Prediction failed".to_string()));

        let mut context = NeuralAnalyticsContext::default();
        context.headset_data = Some(headset_data);
        context.model_service = create_static_mock(mock);

        let command = PredictColorThinkingCommand {};
        let command_bus = setup_command_bus();

        // Act
        let _ = command_bus.execute(&mut context, command).await;
    
        assert!(context.color_thinking.is_empty());
    }
}
