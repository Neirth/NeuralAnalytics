use std::collections::HashMap;

// Este módulo define el puerto (interface) para interactuar con un EEG headset,
// inspirado en las funciones del script de Python. Permite inicializar el hardware,
// cambiar el modo del dispositivo (señal o impedancia), obtener datos y limpiar recursos.


/// Modos de operación del dispositivo, equivalentes a 'signal' e 'impedance'
#[derive(Debug, PartialEq, Eq)]
pub enum DeviceMode {
    Signal,
    Impedance,
}

/// Errores que pueden ocurrir al interactuar con el dispositivo EEG
#[derive(Debug)]
pub enum EegHeadsetError {
    InitializationError(String),
    ModeError(String),
    DataFetchError(String),
    CleanupError(String),
}

/// Estructura para representar los datos EEG capturados de cada canal.
#[derive(Debug, Clone)]
pub struct EegData {
    pub channels: HashMap<String, f64>,
}

impl EegData {
    /// Crea una nueva instancia de EegData con valores iniciales en 0.
    pub fn new() -> Self {
        let mut channels = HashMap::new();
        channels.insert("T3".to_string(), 0.0);
        channels.insert("T4".to_string(), 0.0);
        channels.insert("O1".to_string(), 0.0);
        channels.insert("O2".to_string(), 0.0);
        Self { channels }
    }
}

/// Estructura para representar los datos de impedancia de cada canal.
#[derive(Debug, Clone)]
pub struct ImpedanceData {
    pub channels: HashMap<String, f64>,
}

impl ImpedanceData {
    /// Crea una nueva instancia de ImpedanceData con valores por defecto (ej. 4000 kOhm).
    pub fn new() -> Self {
        let mut channels = HashMap::new();
        channels.insert("T3".to_string(), 4000.0);
        channels.insert("T4".to_string(), 4000.0);
        channels.insert("O1".to_string(), 4000.0);
        channels.insert("O2".to_string(), 4000.0);
        Self { channels }
    }
}

/// Estructura que agrupa los datos de EEG e impedancia.
#[derive(Debug, Clone)]
pub struct DeviceData {
    pub eeg: EegData,
    pub impedance: ImpedanceData,
}

impl DeviceData {
    /// Crea una nueva instancia con datos por defecto.
    pub fn new() -> Self {
        Self {
            eeg: EegData::new(),
            impedance: ImpedanceData::new(),
        }
    }
}

/// Puerto de entrada para el EEG headset.
/// Define la interfaz que debe implementar cualquier adaptador concreto que se
/// conecte al hardware real.
pub trait EegHeadsetPort {
    /// Inicializa el hardware del dispositivo.
    ///
    /// # Retorna
    ///
    /// * Ok(()) si la inicialización es exitosa.
    /// * Err(EegHeadsetError) en caso de error.
    fn initialize(&mut self) -> Result<(), EegHeadsetError>;

    /// Configura el modo de operación del dispositivo, ya sea 'Signal' o 'Impedance'.
    ///
    /// # Argumentos
    ///
    /// * `mode` - Modo al que se desea cambiar el dispositivo.
    ///
    /// # Retorna
    ///
    /// * Ok(()) si se configura correctamente.
    /// * Err(EegHeadsetError) en caso de fallar el cambio de modo.
    fn set_mode(&mut self, mode: DeviceMode) -> Result<(), EegHeadsetError>;

    /// Libera los recursos del dispositivo y cierra la conexión de hardware.
    ///
    /// # Retorna
    ///
    /// * Ok(()) si la limpieza es exitosa.
    /// * Err(EegHeadsetError) en caso de error al liberar recursos.
    fn cleanup(&mut self) -> Result<(), EegHeadsetError>;

    /// Obtiene datos del dispositivo.
    /// La cantidad de muestras a recoger se define por `samples`. La forma de
    /// interpretar estos datos dependerá del modo actual configurado.
    ///
    /// # Argumentos
    ///
    /// * `samples` - Número de muestras o tamaño del paquete de datos a obtener.
    ///
    /// # Retorna
    ///
    /// * Ok(DeviceData) con los datos actuales del dispositivo.
    /// * Err(EegHeadsetError) en caso de error al obtener los datos.
    fn fetch_data(&mut self, samples: usize) -> Result<DeviceData, EegHeadsetError>;
}