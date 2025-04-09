use crate::domain::context::NeuralAnalyticsContext;
use blackbox_di::cell::Ref;
use blackbox_di::{implements, injectable, module};
use presage::{CommandBus, Configuration};

// Import Adapters
use crate::infrastructure::adapters::input::brainbit_headset::BrainFlowAdapter;
use crate::infrastructure::adapters::output::tapo_smartbulb::TapoSmartBulbAdapter;

// Import Use Cases
use crate::domain::use_cases::{
    extract_calibration_use_case::ExtractCalibrationDataUseCase,
    extract_extraction_use_case::ExtractGeneralistDataUseCase,
    initialize_hardware_parts_use_case::InitializeHardwarePartsUseCase,
    predict_color_thinking_use_case::PredictColorThinkingUseCase,
    search_headband_use_case::SearchHeadbandUseCase,
    update_light_status_use_case::UpdateLightStatusUseCase,
};

#[module]
pub struct CoreModule {
    #[provider(dyn EegHeadsetPort)]
    eeg_headset: BrainFlowAdapter,

    #[provider(dyn SmartBulbPort)]
    smart_bulb: TapoSmartBulbAdapter,

    // Provider for the Command Bus (will use the factory below)
    #[provider]
    command_bus_wrapper: InjectableCommandBus,

    // Providers for each Use Case
    #[provider]
    extract_calibration_uc: ExtractCalibrationDataUseCase,
    #[provider]
    extract_generalist_uc: ExtractGeneralistDataUseCase,
    #[provider]
    initialize_hardware_uc: InitializeHardwarePartsUseCase,
    #[provider]
    predict_color_uc: PredictColorThinkingUseCase,
    #[provider]
    search_headband_uc: SearchHeadbandUseCase,
    #[provider]
    update_light_uc: UpdateLightStatusUseCase,
}

#[injectable]
pub struct InjectableCommandBus {
    pub(crate) bus: CommandBus<NeuralAnalyticsContext, presage::Error>,
}

#[implements]
impl InjectableCommandBus {
    #[factory]
    fn new(
        // Inject Ref<T> provided by the container
        extract_calibration_uc: Ref<ExtractCalibrationDataUseCase>,
        extract_generalist_uc: Ref<ExtractGeneralistDataUseCase>,
        initialize_hardware_uc: Ref<InitializeHardwarePartsUseCase>,
        predict_color_uc: Ref<PredictColorThinkingUseCase>,
        search_headband_uc: Ref<SearchHeadbandUseCase>,
        update_light_uc: Ref<UpdateLightStatusUseCase>,
    ) -> Self {
        println!("Factory: Creating CommandBus via Box::leak (WARNING: Leaks memory!)...");

        // Leak the Use Case instances obtained from DI container to get &'static references.
        // This assumes the Use Cases implement Clone. If not, this will fail.
        // Consider the implications of memory leaks carefully.
        let handler1: &'static ExtractCalibrationDataUseCase =
            Box::leak(Box::new((*extract_calibration_uc).clone()));
        let handler2: &'static ExtractGeneralistDataUseCase =
            Box::leak(Box::new((*extract_generalist_uc).clone()));
        let handler3: &'static InitializeHardwarePartsUseCase =
            Box::leak(Box::new((*initialize_hardware_uc).clone()));
        let handler4: &'static PredictColorThinkingUseCase =
            Box::leak(Box::new((*predict_color_uc).clone()));
        let handler5: &'static SearchHeadbandUseCase =
            Box::leak(Box::new((*search_headband_uc).clone()));
        let handler6: &'static UpdateLightStatusUseCase =
            Box::leak(Box::new((*update_light_uc).clone()));

        let bus = CommandBus::<NeuralAnalyticsContext, presage::Error>::new().configure(
            Configuration::new()
                // Pass the obtained &'static references
                .command_handler(handler1)
                .command_handler(handler2)
                .command_handler(handler3)
                .command_handler(handler4)
                .command_handler(handler5)
                .command_handler(handler6),
        );
        println!("Factory: CommandBus created.");
        Self { bus }
    }
}
