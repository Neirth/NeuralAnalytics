"""
Controlador principal para el sistema de captura EEG.
Gestiona la transición entre vistas, la comunicación con el hardware,
y la captura y almacenamiento de datos.
"""

import os
import time
import numpy as np
import threading
import signal
from blessed import Terminal

# Importar componentes del sistema
from config.settings import (
    APP_STATE_SETUP, APP_STATE_COUNTDOWN, APP_STATE_CAPTURE, 
    APP_STATE_COMPLETE, APP_STATE_ERROR, IMPEDANCE_MAP,
    CHANNEL_MAP, INITIAL_DELAY, NUM_SAMPLES, WINDOW_SECONDS
)

from hardware.brainbit import configure_board
from processing.impedance import ohm_to_kohm
from views.view_impedance_data import view_impedance_screen
from views.view_capturer_data import view_capture_screen
from utils.helpers import play_sound

class NeuralCaptureController:
    def __init__(self, scenario_type="red", mac_address=None):
        """
        Inicializa el controlador de captura neural.
        
        Args:
            scenario_type: Tipo de escenario ('red' o 'green')
            mac_address: Dirección MAC del dispositivo BrainBit (opcional)
        """
        self.scenario_type = scenario_type
        self.mac_address = mac_address
        self.board = None
        self.app_state = APP_STATE_SETUP
        self.term = Terminal()
        self.current_device_mode = None  # Para rastrear el modo actual del dispositivo
        
        # Estado compartido
        self.history = {}
        self.state_info = {
            'app_state': APP_STATE_SETUP,
            'message': "Inicializando...",
            'scenario_type': scenario_type,
            'capture_count': 0,
            'countdown_seconds': INITIAL_DELAY
        }
        
        # Eventos de control para hilos
        self.control_event = threading.Event()
        self.data_ready_event = threading.Event()
        
        # Datos actuales (compartidos entre hilos)
        self.current_eeg_values = {electrode: 0 for electrode in ["T3", "T4", "O1", "O2"]}
        self.current_imp_values = {electrode: 0 for electrode in ["T3", "T4", "O1", "O2"]}
        self.lock = threading.Lock()  # Para acceso seguro a datos compartidos
    
    def initialize_hardware(self):
        """Inicializa la conexión con el hardware BrainBit"""
        try:
            self.board = configure_board(self.mac_address)
            self.board.prepare_session()
            self.board.start_stream(450000)
            # No iniciar con ningún modo específico aquí
            return True
        except Exception as e:
            self.state_info['message'] = f"Error al inicializar hardware: {str(e)}"
            self.app_state = APP_STATE_ERROR
            return False
    
    def set_device_mode(self, mode):
        """
        Configura el modo del dispositivo BrainBit.
        
        Args:
            mode: 'signal' para modo EEG, 'impedance' para modo impedancia
        """
        if not self.board:
            return False
        
        # Si ya estamos en el modo solicitado, no hacer nada
        if self.current_device_mode == mode:
            return True
            
        try:
            # Detener el modo actual si existe
            if self.current_device_mode == 'signal':
                self.board.config_board('CommandStopSignal')
            elif self.current_device_mode == 'impedance':
                self.board.config_board('CommandStopResist')
                
            # Activar el nuevo modo
            if mode == 'signal':
                self.board.config_board('CommandStartSignal')
                self.current_device_mode = 'signal'
            elif mode == 'impedance':
                self.board.config_board('CommandStartResist')
                self.current_device_mode = 'impedance'
                
            # Pequeño delay para estabilización
            time.sleep(1)
            return True
        except Exception as e:
            print(f"Error al cambiar modo del dispositivo: {str(e)}")
            return False
    
    def cleanup_hardware(self):
        """Limpia y cierra la conexión con el hardware"""
        if self.board:
            try:
                # Detener el modo activo antes de liberar
                if self.current_device_mode == 'signal':
                    self.board.config_board('CommandStopSignal')
                elif self.current_device_mode == 'impedance':
                    self.board.config_board('CommandStopResist')
                    
                self.board.stop_stream()
                self.board.release_session()
            except Exception as e:
                print(f"Error al liberar sesión: {str(e)}")
    
    def data_provider(self):
        """
        Proveedor de datos para las vistas.
        Retorna los valores actuales de EEG e impedancia.
        """
        with self.lock:
            return self.current_eeg_values.copy(), self.current_imp_values.copy()
    
    def update_data_thread(self):
        """
        Hilo para actualizar datos del dispositivo (EEG e impedancia).
        Se ejecuta en segundo plano y actualiza los valores actuales.
        """
        electrodes = ["T3", "T4", "O1", "O2"]
        last_app_state = None
        
        while not self.control_event.is_set():
            try:
                # Verificar que el board exista y esté listo
                if self.board is None:
                    time.sleep(0.5)
                    continue
                    
                # Verificar si el estado de la aplicación ha cambiado
                if last_app_state != self.app_state:
                    # Cambiar modo del dispositivo según el nuevo estado
                    if self.app_state == APP_STATE_SETUP:
                        self.set_device_mode('impedance')
                    else:  # APP_STATE_COUNTDOWN, APP_STATE_CAPTURE, etc.
                        self.set_device_mode('signal')
                    
                    last_app_state = self.app_state
                    # Dar tiempo adicional para que el dispositivo se estabilice
                    time.sleep(1.0)
                
                # Obtener datos según el modo actual
                if self.current_device_mode == 'impedance':
                    try:
                        # En modo impedancia, obtener valores directamente sin cambiar el modo
                        data = self.board.get_board_data(100)
                        
                        values = {}
                        if data.size > 0:
                            for electrode in electrodes:
                                imp_channel = IMPEDANCE_MAP[electrode]
                                if imp_channel < data.shape[0]:
                                    # Obtener valores absolutos de resistencia en Ohm
                                    resistance_values_ohm = np.abs(data[imp_channel])
                                    
                                    # Convertir a kOhm para el filtrado y procesamiento
                                    resistance_values_kohm = ohm_to_kohm(resistance_values_ohm)
                                    
                                    # Filtrar valores válidos (ahora en kOhm)
                                    valid_values = resistance_values_kohm[(resistance_values_kohm > 1) & (resistance_values_kohm < 5000)]
                                    
                                    if len(valid_values) > 0:
                                        values[electrode] = np.mean(valid_values)
                                    else:
                                        values[electrode] = 4000  # Valor por defecto en kOhm
                                else:
                                    values[electrode] = 4000  # Valor por defecto en kOhm
                        else:
                            values = {electrode: 4000 for electrode in electrodes}
                        
                        with self.lock:
                            self.current_imp_values = values
                        
                    except Exception as e:
                        print(f"Error al obtener datos de impedancia: {str(e)}")
                        time.sleep(0.5)  # Esperar antes de reintentar
                        continue  # Continuar con el siguiente ciclo del bucle
                
                elif self.current_device_mode == 'signal':
                    try:
                        # En modo señal EEG, obtener valores EEG
                        data = self.board.get_board_data(100)
                        
                        # Procesar datos EEG para visualización
                        if data.size > 0:
                            eeg_values = {}
                            for electrode in electrodes:
                                eeg_channel = CHANNEL_MAP[electrode]
                                if eeg_channel < data.shape[0]:
                                    eeg_data = data[eeg_channel]
                                    eeg_values[electrode] = np.mean(np.abs(eeg_data))
                                else:
                                    eeg_values[electrode] = 0
                            
                            with self.lock:
                                self.current_eeg_values = eeg_values
                    
                    except Exception as e:
                        print(f"Error al obtener datos EEG: {str(e)}")
                        time.sleep(0.5)
                        continue
                
                # Indicar que hay nuevos datos disponibles
                self.data_ready_event.set()
                self.data_ready_event.clear()
                
                # Esperar un poco para no sobrecargar el dispositivo
                time.sleep(0.2)
                
            except Exception as e:
                print(f"Error en hilo de datos: {str(e)}")
                time.sleep(1)  # Esperar antes de reintentar
    
    def capture_data_thread(self):
        """
        Hilo para capturar datos EEG en el momento adecuado.
        Solo se ejecuta cuando estamos en estado de captura.
        """
        last_capture_time = 0
        capture_count = 0
        buffer_data = None  # Buffer acumulado de datos
        
        while not self.control_event.is_set():
            try:
                # Solo capturar cuando estemos en estado de captura
                if self.app_state != APP_STATE_CAPTURE:
                    time.sleep(0.5)
                    continue
                    
                current_time = time.time()
                
                # Actualizar contador visible para el usuario
                with self.lock:
                    self.state_info['capture_count'] = capture_count
                
                # Solo capturar un segmento cada WINDOW_SECONDS
                if current_time - last_capture_time >= WINDOW_SECONDS:
                    try:
                        # Obtener datos EEG más completos para el guardado
                        data = self.board.get_board_data(250 * WINDOW_SECONDS)
                        
                        # Seleccionar solo los canales relevantes
                        selected_data = np.take(data, [
                            CHANNEL_MAP["timestamp"] if "timestamp" in CHANNEL_MAP else 10,
                            CHANNEL_MAP["T3"],
                            CHANNEL_MAP["T4"], 
                            CHANNEL_MAP["O1"],
                            CHANNEL_MAP["O2"]
                        ], axis=0).T
                        
                        # Añadir datos al buffer
                        if buffer_data is None:
                            buffer_data = selected_data
                        else:
                            buffer_data = np.vstack((buffer_data, selected_data))
                        
                        print(f"[INFO] Captura {capture_count+1}: Obtenidas {len(selected_data)} filas. Buffer acumulado: {len(buffer_data)} filas")
                        
                        # Mientras tengamos suficientes datos, guardar archivos de 100 filas
                        while len(buffer_data) >= 100:
                            # Preparar directorio para guardar
                            base_dir = f"data/{self.scenario_type}/"
                            os.makedirs(base_dir, exist_ok=True)
                            timestamp = time.strftime("%Y%m%d-%H%M%S")
                            filename = f"{base_dir}{self.scenario_type}_{timestamp}_{capture_count:03d}.csv"
                            
                            # Extraer exactamente 100 filas
                            save_data = buffer_data[:100]
                            # Mantener el resto en el buffer
                            buffer_data = buffer_data[100:]
                            
                            # Guardar en CSV
                            np.savetxt(filename, save_data, delimiter=',',
                                      header="timestamp,T3,T4,O1,O2",
                                      fmt=['%.3f', '%.6f', '%.6f', '%.6f', '%.6f'])
                            
                            print(f"[INFO] Archivo CSV guardado: {os.path.basename(filename)} con exactamente 100 filas")
                            
                            # Incrementar contador y actualizar mensaje
                            capture_count += 1
                            with self.lock:
                                self.state_info['capture_count'] = capture_count
                                self.state_info['message'] = f"Muestra {capture_count}/{NUM_SAMPLES} capturada"
                            
                            # Comprobar si hemos terminado
                            if capture_count >= NUM_SAMPLES:
                                break
                        
                        # Actualizar estado
                        last_capture_time = current_time
                        
                        # Si hemos terminado todas las capturas
                        if capture_count >= NUM_SAMPLES:
                            # Si quedan datos en el buffer y menos de 100 filas, rellenar con ceros
                            if buffer_data is not None and len(buffer_data) > 0:
                                print(f"[INFO] Quedan {len(buffer_data)} filas en el buffer, desechando...")
                            
                            with self.lock:
                                self.app_state = APP_STATE_COMPLETE
                                self.state_info['app_state'] = APP_STATE_COMPLETE
                                self.state_info['message'] = "¡Captura completada! Procesando datos..."
                            break
                            
                    except Exception as e:
                        error_msg = f"Error al capturar datos: {str(e)}"
                        print(f"[ERROR] {error_msg}")
                        
                        # Solo cambiar a error si es un problema grave
                        if "Fatal" in str(e) or capture_count == 0:  # Si es un error fatal o no hemos capturado nada
                            with self.lock:
                                self.app_state = APP_STATE_ERROR
                                self.state_info['app_state'] = APP_STATE_ERROR
                                self.state_info['message'] = error_msg
                            break
                        else:
                            # Si ya tenemos algunas capturas, seguir intentando
                            time.sleep(1)
                
                time.sleep(0.1)
                
            except Exception as e:
                print(f"[ERROR CAPTURA] Error general en hilo de captura: {str(e)}")
                time.sleep(0.5)
        
        # Asegurar que se registre el resultado, sea éxito o fallo
        if capture_count >= NUM_SAMPLES and not self.control_event.is_set():
            print(f"[INFO] Captura completada correctamente. Se guardaron {capture_count} archivos CSV de 100 filas cada uno.")
        else:
            print(f"[INFO] Captura interrumpida o incompleta. Solo se guardaron {capture_count}/{NUM_SAMPLES} archivos.")
    
    def countdown_thread(self):
        """
        Hilo para gestionar la cuenta regresiva antes de la captura.
        """
        start_time = time.time()
        
        while not self.control_event.is_set():
            if self.app_state != APP_STATE_COUNTDOWN:
                time.sleep(0.5)
                continue
                
            # Calcular tiempo restante
            elapsed = time.time() - start_time
            countdown_seconds = max(0, INITIAL_DELAY - int(elapsed))
            
            with self.lock:
                self.state_info['countdown_seconds'] = countdown_seconds
            
            # Si la cuenta regresiva llegó a 0, cambiar a modo captura
            if countdown_seconds <= 0:
                with self.lock:
                    self.app_state = APP_STATE_CAPTURE
                    self.state_info['app_state'] = APP_STATE_CAPTURE
                    self.state_info['message'] = "Captura iniciada. Mantenga la posición indicada."
                
                play_sound("Comienza la captura")
                break
                
            time.sleep(0.1)
    
    def handle_keyboard_input(self):
        """
        Maneja entradas de teclado para controlar la aplicación.
        """
        key = self.term.inkey(timeout=0.1)
        
        if key:
            # Comprobar de múltiples formas si es ENTER (podría ser '\n', '\r' o KEY_ENTER)
            if key.code == self.term.KEY_ENTER or key == '\n' or key == '\r':
                # Avanzar al siguiente estado según el estado actual
                if self.app_state == APP_STATE_SETUP:
                    self.app_state = APP_STATE_COUNTDOWN
                    self.state_info['app_state'] = APP_STATE_COUNTDOWN
                    self.state_info['message'] = f"Preparando captura. La adquisición comenzará en {INITIAL_DELAY} segundos."
                    return {'action': 'next'}
                    
            elif key.code == self.term.KEY_ESCAPE or key == 'q':
                # Cancelar operación
                self.control_event.set()
                return {'action': 'cancel'}
            
            # Agregar impresión de depuración para ver qué tecla se está presionando
            print(f"Tecla presionada: {repr(key)}, código: {key.code if hasattr(key, 'code') else 'sin código'}")
                
        return {'action': 'none'}
    
    def handle_view_event(self, event_data):
        """
        Procesa eventos generados por las vistas.
        
        Args:
            event_data: Diccionario con información del evento
            
        Returns:
            dict: Resultado del procesamiento del evento
        """
        if not event_data or 'event' not in event_data:
            return {'action': 'none'}
        
        # Manejar eventos de teclado
        if event_data['event'] == 'key_press':
            key = event_data.get('key', '')
            
            if key == 'enter':
                # Si es la pantalla de impedancia y la impedancia es correcta
                if self.app_state == APP_STATE_SETUP and event_data.get('impedance_ok', False):
                    self.app_state = APP_STATE_COUNTDOWN
                    self.state_info['app_state'] = APP_STATE_COUNTDOWN
                    self.state_info['message'] = f"Preparando captura. La adquisición comenzará en {INITIAL_DELAY} segundos."
                    return {'action': 'next'}
                    
            elif key == 'escape':
                # Cancelar operación
                self.control_event.set()
                return {'action': 'cancel'}
        
        return {'action': 'none'}
    
    def run(self):
        """
        Método principal para ejecutar el flujo de captura neural.
        Gestiona las transiciones entre estados y vistas.
        """
        # Inicializar hardware
        if not self.initialize_hardware():
            print("[ERROR] Error al inicializar hardware")
            return False
            
        print("[INFO] Hardware inicializado correctamente")
        
        # Señales para capturar Ctrl+C y otros eventos
        def signal_handler(sig, frame):
            print("[INFO] Señal de interrupción recibida")
            self.control_event.set()
        
        signal.signal(signal.SIGINT, signal_handler)
        signal.signal(signal.SIGTERM, signal_handler)
        
        try:
            # Iniciar hilo de actualización de datos en segundo plano
            data_thread = threading.Thread(target=self.update_data_thread)
            data_thread.daemon = True
            data_thread.start()
            print("[INFO] Hilo de actualización de datos iniciado")
            
            # 1. Fase de configuración de impedancia
            with self.term.fullscreen(), self.term.hidden_cursor(), self.term.cbreak():
                while not self.control_event.is_set() and self.app_state == APP_STATE_SETUP:
                    # Mostrar vista de impedancia
                    event_result = view_impedance_screen(
                        self.term, self.board, self.history, self.control_event, self.data_provider)
                    
                    # Procesar evento de la vista
                    if isinstance(event_result, dict) and 'event' in event_result:
                        result = self.handle_view_event(event_result)
                        
                        if result['action'] == 'next':
                            print("[INFO] Avanzando a fase de cuenta regresiva")
                            break
                        elif result['action'] == 'cancel':
                            play_sound("Operación cancelada")
                            print("[INFO] Operación cancelada por usuario")
                            return False
                            
                    elif event_result is False:  # Si la vista devolvió False (error)
                        print("[ERROR] Error en la pantalla de impedancia")
                        return False
                        
                    time.sleep(0.1)
                
                if self.control_event.is_set():
                    print("[INFO] Operación cancelada (evento de control)")
                    return False
                
                # 2. Transición a fase de cuenta regresiva
                self.app_state = APP_STATE_COUNTDOWN
                self.state_info['app_state'] = APP_STATE_COUNTDOWN
                self.state_info['message'] = f"Preparando captura. La adquisición comenzará en {INITIAL_DELAY} segundos."
                print("[INFO] Iniciando cuenta regresiva")
                
                # Iniciar hilo de cuenta regresiva
                countdown_thread = threading.Thread(target=self.countdown_thread)
                countdown_thread.daemon = True
                countdown_thread.start()
                
                # Iniciar hilo de captura de datos
                capture_thread = threading.Thread(target=self.capture_data_thread)
                capture_thread.daemon = True
                capture_thread.start()
                print("[INFO] Hilos de cuenta regresiva y captura iniciados")
                
                # 3. Mostrar vista de captura (cuenta regresiva y captura)
                while not self.control_event.is_set() and self.app_state in [APP_STATE_COUNTDOWN, APP_STATE_CAPTURE, APP_STATE_COMPLETE]:
                    # Mostrar la vista de captura
                    result = view_capture_screen(
                        self.term, self.data_provider, 
                        self.history, self.state_info, self.control_event
                    )
                    
                    # Procesar resultado de la vista
                    if result['action'] == 'cancel':
                        play_sound("Operación cancelada")
                        print("[INFO] Operación cancelada por usuario")
                        return False
                    elif result['action'] == 'error':
                        play_sound("Error en la captura")
                        print(f"[ERROR] Error en la vista de captura: {result.get('message', 'desconocido')}")
                        return False
                    elif result['action'] == 'complete':
                        play_sound("Captura completada")
                        print("[INFO] Captura completada con éxito")
                        return True
                        
                    # Si hemos completado, salir del bucle
                    if self.app_state == APP_STATE_COMPLETE:
                        print("[INFO] Estado completo detectado, finalizando bucle")
                        break
                        
                    time.sleep(0.1)
            
            # Verificar resultado final
            print(f"[INFO] Estado final: {self.app_state}")
            if self.app_state == APP_STATE_COMPLETE:
                play_sound("Captura completada")
                return True
            elif self.app_state == APP_STATE_ERROR:
                play_sound("Error en la captura")
                return False
            else:
                print("[WARN] Estado final inesperado")
                return False
                
        except Exception as e:
            print(f"[ERROR] Error en controlador principal: {str(e)}")
            return False
        finally:
            # Establecer evento de control para detener todos los hilos
            self.control_event.set()
            # Limpiar conexión hardware
            self.cleanup_hardware()
            print("[INFO] Recursos liberados")



