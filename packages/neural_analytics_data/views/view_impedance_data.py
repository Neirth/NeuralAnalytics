"""
Visualización de datos de impedancia para el sistema de captura EEG.
Este módulo maneja la pantalla de configuración inicial donde se muestran
los valores de impedancia para garantizar un buen contacto de los electrodos.
"""

import time
import io
import sys
from collections import deque

from config.settings import (
    APP_STATE_SETUP, APP_STATE_ERROR,
    CHANNEL_MAP, MAX_HISTORY
)

from hardware.brainbit import get_resistance_values
from utils.helpers import get_impedance_status, play_sound
from views.view_layout import view_header, view_electrode_graph, view_status_bar

def view_impedance_screen(term, board, history, control_event=None, data_provider=None):
    """
    Muestra la pantalla de configuración con datos de impedancia de los electrodos.
    
    Args:
        term: Terminal blessed
        board: Objeto BoardShim del dispositivo BrainBit
        history: Diccionario para almacenar el historial de valores
        control_event: Evento para controlar la interrupción del bucle
        data_provider: Función que proporciona los datos actuales (eeg_values, imp_values)
        
    Returns:
        dict: Evento generado por la vista o False en caso de error
    """
    electrodes = ["T3", "T4", "O1", "O2"]
    
    # Inicializar historiales de impedancia si es necesario
    for electrode in electrodes:
        if f"{electrode}_imp" not in history:
            history[f"{electrode}_imp"] = deque(maxlen=MAX_HISTORY)
    
    # Contador para actualización periódica
    if 'update_count' not in history:
        history['update_count'] = 0
    update_count = history['update_count']
    update_count += 1
    history['update_count'] = update_count
    
    try:
        if control_event and control_event.is_set():
            return {'event': 'cancel'}
            
        # Obtener dimensiones actuales de la terminal
        width = term.width
        height = term.height
        
        # Calcular dimensiones de los gráficos basados en el tamaño actual
        graph_width = width // 2 - 4
        graph_height = (height - 14) // 2  # Reducido de 10 a 14 para dejar más espacio
        
        # Calcular margen para mejor centrado
        margin_x = (width - graph_width * 2) // 3
        
        # Posiciones de los gráficos (2x2 grid)
        positions = {
            "T3": (margin_x, 5),
            "T4": (margin_x * 2 + graph_width, 5),
            "O1": (margin_x, 5 + graph_height + 1),  # Reducido de +2 a +1
            "O2": (margin_x * 2 + graph_width, 5 + graph_height + 1)  # Reducido de +2 a +1
        }
        
        # Redirigir stdout temporalmente para capturar salida
        old_stdout = sys.stdout
        buffer = io.StringIO()
        sys.stdout = buffer
        
        # Dibujar todos los elementos en el buffer
        print(term.home + term.clear_eos, end='')
        view_header(term, y_start=1)
        
        # Obtener valores de impedancia
        if data_provider:
            _, resistance_values = data_provider()
        else:
            resistance_values = get_resistance_values(board, electrodes)
            
        # Verificar estado de impedancias
        all_ok = all(get_impedance_status(resistance_values[e])[1] <= 2 for e in electrodes)
        
        # Mensaje según el estado
        if all_ok:
            status_msg = "¡Todos los electrodos tienen buen contacto! Presione ENTER para continuar"
        else:
            status_msg = "Ajuste los electrodos hasta que todos tengan contacto EXCELENTE o ACEPTABLE"
        
        # Dibujar gráficos de impedancia
        for electrode in electrodes:
            value = resistance_values[electrode]
            x, y = positions[electrode]
            
            level = view_electrode_graph(
                term, x, y, graph_width, graph_height,
                electrode, value, history, APP_STATE_SETUP
            )
            
            # Actualizar historial para este electrodo
            if f"{electrode}_imp" in history:
                history[f"{electrode}_imp"].append(value)
        
        # Barra de estado
        view_status_bar(term, height - 7, APP_STATE_SETUP, status_msg)
        
        # Restaurar stdout y enviar todo el buffer de una vez
        sys.stdout = old_stdout
        print(buffer.getvalue(), end='', flush=True)
        
        # Comprobar input del usuario
        key = term.inkey(timeout=0.1, esc_delay=0)

        if key:
            # Imprimir información sobre la tecla
            debug_msg = f"Tecla: {repr(key)}, código: {key.code if hasattr(key, 'code') else 'N/A'}"
            print(term.move_xy(2, height - 2) + term.clear_eol + debug_msg, end='', flush=True)
            
            # ENTER: Continuar solo si la impedancia es correcta
            is_enter = (key.code == term.KEY_ENTER or key == '\n' or key == '\r')
            if is_enter and all_ok:
                print(term.move_xy(2, height - 3) + term.clear_eol + "¡ENTER detectado! Avanzando...", end='', flush=True)
                play_sound("Impedancia correcta, continuar")
                return {'event': 'key_press', 'key': 'enter', 'impedance_ok': True}
            
            # ESC o Q: Cancelar
            elif key.code == term.KEY_ESCAPE or key == 'q' or key == 'Q':
                play_sound("Operación cancelada")
                return {'event': 'key_press', 'key': 'escape'}
        
    except Exception as e:
        print(term.move_xy(1, 1) + term.clear + term.bold_red(f"Error: {str(e)}"))
        time.sleep(3)
        return False
        
    return None