"""
Configuración para el sistema de captura EEG.
"""

# Parámetros de captura
NUM_SAMPLES = 100              # Número total de muestras a capturar
WINDOW_SECONDS = 10            # Duración de cada ventana de captura (segundos)
INITIAL_DELAY = 30             # Retraso inicial antes de comenzar la captura (segundos)
BEEP_FREQ = 1000               # Frecuencia del sonido de alerta (Hz)
MAX_HISTORY = 50               # Número máximo de puntos en el historial de gráficas

# Configuración del dispositivo BrainBit
BRAINBIT_MAC_DEFAULT = "C8:8F:B6:6D:E1:E2"  # Dirección MAC por defecto

# Mapeo de canales EEG
CHANNEL_MAP = {
    "timestamp": 10,
    "T3": 1,
    "T4": 2,
    "O1": 3,
    "O2": 4
}

# Mapeo de canales de impedancia
IMPEDANCE_MAP = {
    "T3": 5,
    "T4": 6,
    "O1": 7,
    "O2": 8
}

# Límites de impedancia (kΩ)
IMPEDANCIA_TOO_LOW = 200      # Impedancia mínima aceptable (kΩ)
IMPEDANCE_EXCELLENT = 800     # Impedancia en contacto óptimo (kΩ)
IMPEDANCE_ACCEPTABLE = 1500   # Impedancia aceptable (kΩ)
IMPEDANCE_POOR = 2000         # Impedancia con mala conexión (kΩ)

# Estados de la aplicación
APP_STATE_SETUP = "setup"         # Configuración/inicialización del casco
APP_STATE_COUNTDOWN = "countdown"  # Cuenta regresiva previa a la captura
APP_STATE_CAPTURE = "capture"      # Captura de datos EEG
APP_STATE_COMPLETE = "complete"    # Captura completada
APP_STATE_ERROR = "error"          # Error en la captura

if __name__ == "__main__":
    # Prueba simple de la configuración
    print("Configuración del sistema de captura EEG:")
    print(f"NUM_SAMPLES = {NUM_SAMPLES}")
    print(f"WINDOW_SECONDS = {WINDOW_SECONDS}")
    print(f"INITIAL_DELAY = {INITIAL_DELAY}")
    print(f"BRAINBIT_MAC_DEFAULT = {BRAINBIT_MAC_DEFAULT}")
