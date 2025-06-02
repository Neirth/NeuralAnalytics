use log::{debug, error, info};
use presage::{CommandBus, Configuration, Event};
use statig::prelude::*;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use crate::{
    domain::{
        commands::{
            disconnect_headband_command::DisconnectHeadbandCommand,
            extract_calibration_data_command::ExtractCalibrationDataCommand,
            extract_generalist_data_command::ExtractGeneralistDataCommand,
            predict_color_thinking_command::PredictColorThinkingCommand,
            search_headband_command::SearchHeadbandCommand,
            update_light_status_command::UpdateLightStatusCommand,
        },
        context::NeuralAnalyticsContext,
        events::{
            captured_headset_data_event::CapturedHeadsetDataEvent,
            headset_calibrated_event::HeadsetCalibratedEvent,
            headset_calibrating_event::HeadsetCalibratingEvent,
            headset_connected_event::HeadsetConnectedEvent,
            headset_disconnected_event::HeadsetDisconnectedEvent,
            initialized_core_event::InitializedCoreEvent,
        },
        use_cases::{
            disconnect_headband_use_case::disconnect_headband_use_case,
            extract_calibration_use_case::extract_calibration_data_use_case,
            extract_extraction_use_case::extract_generalist_data_use_case,
            predict_color_thinking_use_case::predict_color_thinking_use_case,
            search_headband_use_case::search_headband_use_case,
            update_light_status_use_case::update_light_status_use_case,
        },
    },
    utils::send_event,
    EventData,
};

use super::neural_events::NeuralAnalyticsCoreEvents;

/// Main state machine - Initializes and holds DI container internally.
pub(crate) struct MainStateMachine {
    context: Arc<Mutex<NeuralAnalyticsContext>>,
    command_bus: CommandBus<NeuralAnalyticsContext, presage::Error>,
}

#[state_machine(initial = "State::initialize_application()", state(derive(Debug)))]
impl MainStateMachine {
    /// Creates a new instance of the MainStateMachine asynchronously,
    /// building the necessary DI container.
    pub async fn new() -> Self {
        debug!("Initializate state machine...");

        let bus = CommandBus::<NeuralAnalyticsContext, presage::Error>::new().configure(
            Configuration::new()
                .command_handler(&disconnect_headband_use_case)
                .command_handler(&extract_calibration_data_use_case)
                .command_handler(&extract_generalist_data_use_case)
                .command_handler(&predict_color_thinking_use_case)
                .command_handler(&search_headband_use_case)
                .command_handler(&update_light_status_use_case),
        );

        Self {
            context: Arc::new(Mutex::new(NeuralAnalyticsContext::default())),
            command_bus: bus,
        }
    }

    /// Initialization state for the Neural Analytics system.
    /// This state initializes all necessary hardware components
    /// and prepares the system for operation.
    ///
    /// # State Flow
    /// - Executes `InitializeHardwarePartsCommand`
    /// - Emits `InitializedCoreEvent` upon successful initialization
    /// - Transitions to `awaiting_headset_connection` state
    #[state]
    #[allow(unused_variables)]
    async fn initialize_application(
        &mut self,
        event: &NeuralAnalyticsCoreEvents,
    ) -> Response<State> {
        // Initialization state - Detailed logging
        debug!("Executing state: initialize_application");

        if let Err(e) = send_event(
            &InitializedCoreEvent::NAME.to_string(),
            &EventData::default(),
        ) {
            error!("Failed to send initialized core event: {}", e);
            debug!("Repeating state: initialize_application due to error");
            return Transition(State::initialize_application());
        }

        debug!("Transitioning to state: awaiting_headset_connection");

        // Direct transition to the next state
        Transition(State::awaiting_headset_connection())
    }

    /// State that waits for a headset to connect to the system.
    /// This state continuously polls for available headsets
    /// and transitions to the calibration state when a connection is established.
    ///
    /// # State Flow
    /// - Executes `SearchHeadbandCommand` to find connected devices
    /// - Emits either `HeadsetConnectedEvent` or `HeadsetDisconnectedEvent`
    /// - On connection success, transitions to `awaiting_headset_calibration`
    /// - On connection failure, remains in `awaiting_headset_connection`
    #[state]
    #[allow(unused_variables)]
    async fn awaiting_headset_connection(
        &mut self,
        event: &NeuralAnalyticsCoreEvents,
    ) -> Response<State> {
        debug!("Executing state: awaiting_headset_connection");
        debug!("Disconnecting headset...");

        let disconnect_result = {
            let mut ctx = self.context.lock().await;
            self.command_bus
                .execute(&mut *ctx, DisconnectHeadbandCommand)
                .await
        };

        info!("Searching for headset...");

        let search_result = {
            let mut ctx = self.context.lock().await;
            self.command_bus
                .execute(&mut *ctx, SearchHeadbandCommand)
                .await
        };

        match search_result {
            Ok(_) => {
                // Headset connected
                info!("Headset correctly connected");
                if let Err(e) = send_event(
                    &HeadsetConnectedEvent::NAME.to_string(),
                    &EventData::default(),
                ) {
                    error!("Failed to send headset connected event: {}", e);

                    Transition(State::awaiting_headset_connection())
                } else {
                    debug!("Transitioning to state: awaiting_headset_calibration");
                    Transition(State::awaiting_headset_calibration())
                }
            }
            Err(_) => {
                // Headset disconnected
                info!("Headset not connected");

                if let Err(e) = send_event(
                    &HeadsetDisconnectedEvent::NAME.to_string(),
                    &EventData::default(),
                ) {
                    error!("Failed to send headset disconnected event: {}", e);
                }

                Transition(State::awaiting_headset_connection())
            }
        }
    }

    /// State that handles the headset calibration process.
    /// This state verifies that the headset's impedance levels are
    /// within acceptable ranges before allowing data capture.
    ///
    /// # State Flow
    /// - Executes `ExtractCalibrationDataCommand` to obtain impedance data
    /// - Analyzes impedance values to determine if calibration is acceptable
    /// - If calibration fails due to connection issues, returns to `awaiting_headset_connection`
    /// - If impedance values are too high (> 1000), emits `HeadsetCalibratingEvent` and remains in this state
    /// - If impedance values are acceptable, transitions to `capturing_headset_data`
    #[state]
    #[allow(unused_variables)]
    async fn awaiting_headset_calibration(
        &mut self,
        event: &NeuralAnalyticsCoreEvents,
    ) -> Response<State> {
        // Send debug message
        debug!("Executing state: awaiting_headset_calibration");

        // Get calibration data from internal context
        let calibration_result = {
            let mut ctx = self.context.lock().await;
            self.command_bus
                .execute(&mut *ctx, ExtractCalibrationDataCommand)
                .await
        };

        if calibration_result.is_err() {
            if let Err(e) = send_event(
                &HeadsetDisconnectedEvent::NAME.to_string(),
                &EventData::default(),
            ) {
                error!("Failed to send headset disconnected event: {}", e);
            }

            return Transition(State::awaiting_headset_connection());
        }

        // Get impedance data from internal context
        let impedance_data = {
            let ctx = self.context.lock().await;
            ctx.impedance_data.clone()
        };

        if let Some(data) = impedance_data {
            let needs_more_calibration = data.values().any(|&value| value > 1000 || value < 1);

            if needs_more_calibration {
                if let Err(e) = send_event(
                    &HeadsetCalibratingEvent::NAME.to_string(),
                    &EventData {
                        impedance_data: Some(data),
                        ..Default::default()
                    },
                ) {
                    error!("Failed to send headset calibrating event: {}", e);
                }

                return Transition(State::awaiting_headset_calibration());
            }
        }

        // If we get here, the device is calibrated
        if let Err(e) = send_event(
            &HeadsetCalibratedEvent::NAME.to_string(),
            &EventData::default(),
        ) {
            error!("Failed to send headset calibrated event: {}", e);
        }

        Transition(State::capturing_headset_data())
    }

    /// State for capturing and processing neural data from the headset.
    /// This state continuously retrieves EEG data, runs it through the
    /// machine learning model for color prediction, and controls output devices.
    ///
    /// # State Flow
    /// - Executes `ExtractGeneralistDataCommand` to get raw EEG data
    /// - If data extraction fails, returns to `awaiting_headset_connection`
    /// - Runs `PredictColorThinkingCommand` to process the data
    /// - Controls light status based on prediction ("green" = on)
    /// - Emits `CapturedHeadsetDataEvent` with processed data
    /// - Continues in this state in a loop to capture more data
    #[state]
    #[allow(unused_variables)]
    async fn capturing_headset_data(
        &mut self,
        event: &NeuralAnalyticsCoreEvents,
    ) -> Response<State> {
        // Start measuring total time
        let start_total = Instant::now();

        // Measure data extraction time
        let start_extraction = Instant::now();
        let extract_result = {
            let mut ctx = self.context.lock().await;
            self.command_bus
                .execute(&mut *ctx, ExtractGeneralistDataCommand)
                .await
        };
        let extraction_time = start_extraction.elapsed();
        info!("Data extraction time: {:?}", extraction_time);

        if extract_result.is_err() {
            if let Err(e) = send_event(
                &HeadsetDisconnectedEvent::NAME.to_string(),
                &EventData::default(),
            ) {
                error!("Failed to send headset disconnected event: {}", e);
            }

            return Transition(State::awaiting_headset_connection());
        }

        let raw_data = {
            let ctx = self.context.lock().await;
            ctx.headset_data.clone().unwrap_or_default()
        };

        // Measure color prediction time (the most computationally intensive part)
        let start_prediction = Instant::now();

        let color_prediction = {
            let mut ctx = self.context.lock().await;
            let prediction_result = self
                .command_bus
                .execute(&mut *ctx, PredictColorThinkingCommand {})
                .await;

            if let Err(e) = prediction_result {
                error!("Failed to predict color thinking: {:?}", e);
                let prediction_time = start_prediction.elapsed();

                if e.to_string().contains("has no data") {
                    if let Err(e) = send_event(
                        &HeadsetDisconnectedEvent::NAME.to_string(),
                        &EventData::default(),
                    ) {
                        error!("Failed to send headset disconnected event: {}", e);
                    }

                    return Transition(State::awaiting_headset_connection());
                } else {
                    return Transition(State::capturing_headset_data());
                }
            }

            ctx.get_color_thinking()
        };
        let prediction_time = start_prediction.elapsed();
        info!("Color prediction time: {:?}", prediction_time);

        // Measure light status update time
        let start_light_update = Instant::now();
        if !color_prediction.is_empty() {
            let is_green = color_prediction == "green";
            let mut ctx = self.context.lock().await;

            if let Err(e) = self
                .command_bus
                .execute(
                    &mut *ctx,
                    UpdateLightStatusCommand {
                        is_light_on: is_green,
                    },
                )
                .await
            {
                error!("Failed to update light status: {:?}", e);
            }
        }
        let light_update_time = start_light_update.elapsed();
        info!("Light update time: {:?}", light_update_time);

        // Measure event sending time
        let start_event_send = Instant::now();
        if let Err(e) = send_event(
            &CapturedHeadsetDataEvent::NAME.to_string(),
            &EventData {
                headset_data: Some(raw_data),
                color_thinking: Some(color_prediction),
                impedance_data: None,
            },
        ) {
            error!("Failed to send captured headset data event: {}", e);
        }
        let event_send_time = start_event_send.elapsed();
        info!("Event sending time: {:?}", event_send_time);

        // Total time
        let total_time = start_total.elapsed();
        info!("Total sample processing time: {:?}", total_time);

        Transition(State::capturing_headset_data())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        models::{bulb_state::BulbState, eeg_work_modes::WorkMode},
        ports::{input::eeg_headset::EegHeadsetPort, output::smart_bulb::SmartBulbPort},
        services::model_inference_service::ModelInferenceInterface,
    };
    use mockall::{mock, predicate::*};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tokio::test;

    // Mocks para los tests
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

    mock! {
        SmartBulbAdapter {}
        #[async_trait::async_trait]
        impl SmartBulbPort for SmartBulbAdapter {
            async fn change_state(&self, state: BulbState) -> Result<(), String>;
        }
    }

    mock! {
        ModelService {}
        impl ModelInferenceInterface for ModelService {
            fn predict_color(&self, data: &HashMap<String, Vec<f32>>) -> Result<String, String>;
            fn is_model_loaded(&self) -> bool;
        }
    }

    /// Helper para crear una referencia estática para EegHeadsetPort
    fn create_static_eeg_mock<T>(
        mock: T,
    ) -> &'static Arc<RwLock<Box<dyn EegHeadsetPort + Send + Sync>>>
    where
        T: EegHeadsetPort + Send + Sync + 'static,
    {
        Box::leak(Box::new(Arc::new(RwLock::new(
            Box::new(mock) as Box<dyn EegHeadsetPort + Send + Sync>
        ))))
    }

    /// Helper para crear una referencia estática para SmartBulbPort
    fn create_static_bulb_mock<T>(
        mock: T,
    ) -> &'static Arc<RwLock<Box<dyn SmartBulbPort + Send + Sync>>>
    where
        T: SmartBulbPort + Send + Sync + 'static,
    {
        Box::leak(Box::new(Arc::new(RwLock::new(
            Box::new(mock) as Box<dyn SmartBulbPort + Send + Sync>
        ))))
    }

    /// Helper para crear una referencia estática para ModelInferenceInterface
    fn create_static_model_mock<T>(
        mock: T,
    ) -> &'static Arc<RwLock<Box<dyn ModelInferenceInterface + Send + Sync>>>
    where
        T: ModelInferenceInterface + Send + Sync + 'static,
    {
        Box::leak(Box::new(Arc::new(RwLock::new(
            Box::new(mock) as Box<dyn ModelInferenceInterface + Send + Sync>
        ))))
    }

    // Helper para crear una máquina de estados para pruebas con mocks configurados
    async fn create_test_state_machine(
        eeg_mock: MockEegHeadsetAdapter,
        bulb_mock: MockSmartBulbAdapter,
        model_mock: MockModelService,
    ) -> MainStateMachine {
        let mut context = NeuralAnalyticsContext::default();

        // Crear referencias estáticas para los mocks
        context.eeg_headset_adapter = create_static_eeg_mock(eeg_mock);
        context.smart_bulb_adapter = create_static_bulb_mock(bulb_mock);
        context.model_service = create_static_model_mock(model_mock);

        // Creamos la máquina de estados con el contexto mockeado
        let bus = CommandBus::<NeuralAnalyticsContext, presage::Error>::new().configure(
            Configuration::new()
                .command_handler(&disconnect_headband_use_case)
                .command_handler(&extract_calibration_data_use_case)
                .command_handler(&extract_generalist_data_use_case)
                .command_handler(&predict_color_thinking_use_case)
                .command_handler(&search_headband_use_case)
                .command_handler(&update_light_status_use_case),
        );

        MainStateMachine {
            context: Arc::new(Mutex::new(context)),
            command_bus: bus,
        }
    }

    // #[test]
    // async fn test_initialize_application_state_transition() {
    //     // Arrange
    //     let eeg_mock = MockEegHeadsetAdapter::new();
    //     let bulb_mock = MockSmartBulbAdapter::new();
    //     let model_mock = MockModelService::new();

    //     // Configuramos el entorno para que send_event tenga éxito
    //     let mut state_machine = create_test_state_machine(eeg_mock, bulb_mock, model_mock).await;

    //     // Act - Ejecutar el estado de inicialización
    //     let result = state_machine
    //         .initialize_application(&NeuralAnalyticsCoreEvents::InitializeCore)
    //         .await;

    //     // Assert - Verificar que transitamos al estado de espera de conexión
    //     if let Response::Transition(State::AwaitingHeadsetConnection { .. }) = result {
    //         // Transición exitosa
    //         assert!(true);
    //     } else {
    //         panic!("Expected transition to awaiting_headset_connection state");
    //     }
    // }

    // #[test]
    // async fn test_awaiting_headset_connection_success() {
    //     // Arrange
    //     let mut eeg_mock = MockEegHeadsetAdapter::new();
    //     eeg_mock.expect_disconnect().returning(|| Ok(()));
    //     eeg_mock.expect_is_connected().returning(|| false); // No conectado inicialmente
    //     eeg_mock.expect_connect().returning(|| Ok(())); // Conexión exitosa
    //     eeg_mock.expect_is_connected().returning(|| true); // Conectado después

    //     let bulb_mock = MockSmartBulbAdapter::new();
    //     let model_mock = MockModelService::new();

    //     let mut state_machine = create_test_state_machine(eeg_mock, bulb_mock, model_mock).await;

    //     // Act
    //     let result = state_machine
    //         .awaiting_headset_connection(&NeuralAnalyticsCoreEvents::BackgroundTick)
    //         .await;

    //     // Assert - Verificar que transitamos al estado de calibración
    //     if let Response::Transition(State::AwaitingHeadsetCalibration { .. }) = result {
    //         // Transición exitosa
    //         assert!(true);
    //     } else {
    //         panic!("Expected transition to awaiting_headset_calibration state");
    //     }
    // }

    #[test]
    async fn test_awaiting_headset_connection_failure() {
        // Arrange
        let mut eeg_mock = MockEegHeadsetAdapter::new();
        eeg_mock.expect_disconnect().returning(|| Ok(()));
        eeg_mock.expect_is_connected().returning(|| false);
        eeg_mock
            .expect_connect()
            .returning(|| Err("Connection failed".to_string()));

        let bulb_mock = MockSmartBulbAdapter::new();
        let model_mock = MockModelService::new();

        let mut state_machine = create_test_state_machine(eeg_mock, bulb_mock, model_mock).await;

        // Act
        let result = state_machine
            .awaiting_headset_connection(&NeuralAnalyticsCoreEvents::BackgroundTick)
            .await;

        // Assert - Verificar que permanecemos en el mismo estado
        if let Response::Transition(State::AwaitingHeadsetConnection { .. }) = result {
            // Se mantiene en el mismo estado (esperado)
            assert!(true);
        } else {
            panic!("Expected to remain in awaiting_headset_connection state");
        }
    }

    #[test]
    async fn test_awaiting_headset_calibration_success() {
        // Arrange
        let mut eeg_mock = MockEegHeadsetAdapter::new();

        let mut impedance_data = HashMap::new();
        impedance_data.insert("sensor1".to_string(), 100);
        impedance_data.insert("sensor2".to_string(), 100);

        eeg_mock
            .expect_extract_impedance_data()
            .returning(move || Ok(impedance_data.clone()));

        eeg_mock.expect_is_connected().returning(|| true);

        eeg_mock.expect_get_work_mode().return_const(WorkMode::Calibration);

        let bulb_mock = MockSmartBulbAdapter::new();
        let model_mock = MockModelService::new();

        let mut state_machine = create_test_state_machine(eeg_mock, bulb_mock, model_mock).await;

        // Configurar los datos de impedancia en el contexto
        {
            let mut ctx = state_machine.context.lock().await;
            let mut data = HashMap::new();
            data.insert("sensor1".to_string(), 100);
            data.insert("sensor2".to_string(), 100);
            ctx.impedance_data = Some(data);
        }

        // Act
        let result = state_machine
            .awaiting_headset_calibration(&NeuralAnalyticsCoreEvents::BackgroundTick)
            .await;

        // Assert - Verificar que transitamos al estado de captura de datos
        if let Response::Transition(State::CapturingHeadsetData { .. }) = result {
            // Transición exitosa
            assert!(true);
        } else {
            panic!("Expected transition to capturing_headset_data state");
        }
    }

    #[test]
    async fn test_awaiting_headset_calibration_needs_more_calibration() {
        // Arrange
        let mut eeg_mock = MockEegHeadsetAdapter::new();

        let mut impedance_data = HashMap::new();
        impedance_data.insert("sensor1".to_string(), 2000); // Valor muy alto, requiere más calibración
        impedance_data.insert("sensor2".to_string(), 100);

        eeg_mock
            .expect_extract_impedance_data()
            .returning(move || Ok(impedance_data.clone()));

        eeg_mock.expect_is_connected().returning(|| true);

        eeg_mock.expect_get_work_mode().return_const(WorkMode::Calibration);

        let bulb_mock = MockSmartBulbAdapter::new();
        let model_mock = MockModelService::new();

        let mut state_machine = create_test_state_machine(eeg_mock, bulb_mock, model_mock).await;

        // Configurar los datos de impedancia en el contexto
        {
            let mut ctx = state_machine.context.lock().await;
            let mut data = HashMap::new();
            data.insert("sensor1".to_string(), 2000);
            data.insert("sensor2".to_string(), 100);
            ctx.impedance_data = Some(data);
        }

        // Act
        let result = state_machine
            .awaiting_headset_calibration(&NeuralAnalyticsCoreEvents::BackgroundTick)
            .await;

        // Assert - Verificar que permanecemos en el mismo estado
        if let Response::Transition(State::AwaitingHeadsetCalibration { .. }) = result {
            // Se mantiene en el estado de calibración (esperado)
            assert!(true);
        } else {
            panic!("Expected to remain in awaiting_headset_calibration state");
        }
    }

    #[test]
    async fn test_awaiting_headset_calibration_fails() {
        // Arrange
        let mut eeg_mock = MockEegHeadsetAdapter::new();

        eeg_mock
            .expect_extract_impedance_data()
            .returning(|| Err("Failed to extract impedance data".to_string()));

        eeg_mock.expect_is_connected().returning(|| true);

        eeg_mock.expect_get_work_mode().return_const(WorkMode::Calibration);

        let bulb_mock = MockSmartBulbAdapter::new();
        let model_mock = MockModelService::new();

        let mut state_machine = create_test_state_machine(eeg_mock, bulb_mock, model_mock).await;

        // Act
        let result = state_machine
            .awaiting_headset_calibration(&NeuralAnalyticsCoreEvents::BackgroundTick)
            .await;

        // Assert - Verificar que volvemos al estado de espera de conexión
        if let Response::Transition(State::AwaitingHeadsetConnection { .. }) = result {
            // Transición al estado de conexión (esperado)
            assert!(true);
        } else {
            panic!("Expected transition to awaiting_headset_connection state");
        }
    }

    #[test]
    async fn test_capturing_headset_data_success() {
        // Arrange
        let mut eeg_mock = MockEegHeadsetAdapter::new();

        let mut raw_data = HashMap::new();
        raw_data.insert("sensor1".to_string(), vec![1.0, 2.0, 3.0]);
        raw_data.insert("sensor2".to_string(), vec![4.0, 5.0, 6.0]);

        eeg_mock
            .expect_extract_raw_data()
            .returning(move || Ok(raw_data.clone()));

        eeg_mock.expect_is_connected().returning(|| true);

        eeg_mock.expect_get_work_mode().return_const(WorkMode::Extraction);

        let mut bulb_mock = MockSmartBulbAdapter::new();
        bulb_mock
            .expect_change_state()
            .with(eq(BulbState::BulbOn))
            .returning(|_| Ok(()));

        let mut model_mock = MockModelService::new();
        model_mock
            .expect_predict_color()
            .returning(|_| Ok("green".to_string()));

        let mut state_machine = create_test_state_machine(eeg_mock, bulb_mock, model_mock).await;

        // Configurar datos en el contexto
        {
            let mut ctx = state_machine.context.lock().await;
            let mut data = HashMap::new();
            data.insert("sensor1".to_string(), vec![1.0, 2.0, 3.0]);
            data.insert("sensor2".to_string(), vec![4.0, 5.0, 6.0]);
            ctx.headset_data = Some(data);
        }

        // Act
        let result = state_machine
            .capturing_headset_data(&NeuralAnalyticsCoreEvents::BackgroundTick)
            .await;

        // Assert - Verificar que permanecemos en el mismo estado
        if let Response::Transition(State::CapturingHeadsetData { .. }) = result {
            // Se mantiene en el mismo estado (esperado)
            assert!(true);
        } else {
            panic!("Expected to remain in capturing_headset_data state");
        }
    }

    #[test]
    async fn test_capturing_headset_data_extraction_fails() {
        // Arrange
        let mut eeg_mock = MockEegHeadsetAdapter::new();

        eeg_mock
            .expect_extract_raw_data()
            .returning(|| Err("Failed to extract data".to_string()));

        eeg_mock.expect_is_connected().returning(|| true);
        eeg_mock.expect_get_work_mode().return_const(WorkMode::Extraction);

        let bulb_mock = MockSmartBulbAdapter::new();
        let model_mock = MockModelService::new();

        let mut state_machine = create_test_state_machine(eeg_mock, bulb_mock, model_mock).await;

        // Act
        let result = state_machine
            .capturing_headset_data(&NeuralAnalyticsCoreEvents::BackgroundTick)
            .await;

        // Assert - Verificar que volvemos al estado de espera de conexión
        if let Response::Transition(State::AwaitingHeadsetConnection { .. }) = result {
            // Transición al estado de conexión (esperado)
            assert!(true);
        } else {
            panic!("Expected transition to awaiting_headset_connection state");
        }
    }

    #[test]
    async fn test_capturing_headset_data_prediction_fails() {
        // Arrange
        let mut eeg_mock = MockEegHeadsetAdapter::new();

        let mut raw_data = HashMap::new();
        raw_data.insert("sensor1".to_string(), vec![1.0, 2.0, 3.0]);

        eeg_mock
            .expect_extract_raw_data()
            .returning(move || Ok(raw_data.clone()));

        eeg_mock.expect_is_connected().returning(|| true);

        eeg_mock.expect_get_work_mode().return_const(WorkMode::Extraction);

        let bulb_mock = MockSmartBulbAdapter::new();

        let mut model_mock = MockModelService::new();
        model_mock
            .expect_predict_color()
            .returning(|_| Err("Model has no data".to_string()));

        let mut state_machine = create_test_state_machine(eeg_mock, bulb_mock, model_mock).await;

        // Configurar datos en el contexto
        {
            let mut ctx = state_machine.context.lock().await;
            let mut data = HashMap::new();
            data.insert("sensor1".to_string(), vec![1.0, 2.0, 3.0]);
            ctx.headset_data = Some(data);
        }

        // Act
        let result = state_machine
            .capturing_headset_data(&NeuralAnalyticsCoreEvents::BackgroundTick)
            .await;

        // Assert - Verificar que volvemos al estado de espera de conexión
        if let Response::Transition(State::AwaitingHeadsetConnection { .. }) = result {
            // Transición al estado de conexión (esperado)
            assert!(true);
        } else {
            panic!("Expected transition to awaiting_headset_connection state");
        }
    }
}
