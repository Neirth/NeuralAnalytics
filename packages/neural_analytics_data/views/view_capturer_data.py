"""
Visualización de datos durante la captura EEG.
Este módulo maneja la presentación de datos EEG durante las fases de
cuenta regresiva, captura y finalización.
"""

import time
from collections import deque

from config.settings import (
    APP_STATE_COUNTDOWN, APP_STATE_CAPTURE, APP_STATE_COMPLETE, APP_STATE_ERROR,
    CHANNEL_MAP, MAX_HISTORY, INITIAL_DELAY, NUM_SAMPLES
)

from utils.helpers import play_sound
from views.view_layout import view_header, view_electrode_graph, view_status_bar

def view_capture_screen(term, data_provider, history, state_info, control_event=None):
    """
    Visualiza la pantalla de captura de datos EEG.
    
    Args:
        term: Terminal blessed
        data_provider: Función que proporciona los datos actuales
        history: Diccionario para almacenar el historial de valores
        state_info: Diccionario con información del estado actual
        control_event: Evento para controlar la interrupción del bucle
    
    Returns:
        dict: Evento generado por la vista
    """
    electrodes = ["T3", "T4", "O1", "O2"]
    
    # Inicializar historiales de EEG si es necesario
    for electrode in electrodes:
        if f"{electrode}_eeg" not in history:
            history[f"{electrode}_eeg"] = deque(maxlen=MAX_HISTORY)
    
    try:
        if control_event and control_event.is_set():
            return {'action': 'cancel'}
            
        # Obtener dimensiones actuales de la terminal
        width = term.width
        height = term.height
        
        # Calcular dimensiones de los gráficos basadas en el tamaño actual
        graph_width = width // 2 - 4
        graph_height = (height - 14) // 2  # Reducido de -10 a -14 para mayor consistencia
        
        # Calcular margen lateral para centrado
        margin_x = (width - graph_width * 2) // 3
        
        # Posiciones de los gráficos (2x2 grid) con mejor distribución
        positions = [
            (margin_x, 5, graph_width, graph_height),                         
            (margin_x * 2 + graph_width, 5, graph_width, graph_height),       
            (margin_x, 5 + graph_height + 1, graph_width, graph_height),      # Reducido de +2 a +1
            (margin_x * 2 + graph_width, 5 + graph_height + 1, graph_width, graph_height)  # Reducido de +2 a +1
        ]
            
        # Limpiar pantalla
        print(term.clear)
        
        # Obtener estado actual del controlador
        app_state = state_info['app_state']
        message = state_info['message']
        capture_count = state_info.get('capture_count', 0)
        countdown_seconds = state_info.get('countdown_seconds', 0)
        
        # Mostrar encabezado con tipo de escenario
        view_header(term, y_start=1, scenario_type=state_info.get('scenario_type'))
        
        # Obtener datos actuales del controlador
        eeg_values, imp_values = data_provider()
        
        # Dibujar gráficos para cada electrodo
        for i, electrode in enumerate(electrodes):
            x, y, w, h = positions[i]
            imp_value = imp_values.get(electrode, 0)
            eeg_value = eeg_values.get(electrode, 0)
            
            view_electrode_graph(
                term, x, y, w, h,
                electrode, imp_value, history,
                app_state, eeg_value
            )
        
        # Configuración para barras de progreso según estado
        countdown = None
        progress = None
        
        if app_state == APP_STATE_COUNTDOWN:
            countdown = (countdown_seconds, INITIAL_DELAY)
        elif app_state == APP_STATE_CAPTURE:
            progress = (capture_count, NUM_SAMPLES)
        
        # Mostrar barra de estado
        view_status_bar(term, height - 7, app_state, message, countdown, progress)  # Cambiado de -5 a -7
        
        # Comprobar input del usuario
        inp = term.inkey(timeout=0.1)
        
        # Si presiona ESC, cancela la operación
        if inp.code == term.KEY_ESCAPE:
            return {'action': 'cancel', 'app_state': APP_STATE_ERROR}
            
        # Si hemos completado o hay error, devolver el control
        if app_state in [APP_STATE_COMPLETE, APP_STATE_ERROR]:
            time.sleep(0.5)  # Mostrar mensaje final brevemente
            return {'action': 'complete', 'app_state': app_state}
            
    except Exception as e:
        print(term.move_xy(2, height - 2) + term.clear_eol + f"Error en visualización: {str(e)}")
        return {'action': 'error', 'app_state': APP_STATE_ERROR, 'message': str(e)}
    
    # Si no hay eventos especiales, devolver normal_update
    return {'action': 'normal_update'}