"""
Visualización de datos de impedancia para el sistema de captura EEG.
Este módulo maneja la pantalla de configuración inicial donde se muestran
los valores de impedancia para garantizar un buen contacto de los electrodos.
"""

import time
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
    
    # Contador para actualización periódica (estado estático entre renderizados)
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
        graph_height = (height - 10) // 2
        
        # Calcular margen para mejor centrado
        margin_x = (width - graph_width * 2) // 3
        
        # Posiciones de los gráficos (2x2 grid)
        positions = {
            "T3": (margin_x, 5),
            "T4": (margin_x * 2 + graph_width, 5),
            "O1": (margin_x, 5 + graph_height + 2),
            "O2": (margin_x * 2 + graph_width, 5 + graph_height + 2)
        }
            
        # Limpiar pantalla
        print(term.clear)
        
        # Mostrar encabezado
        view_header(term, y_start=1)
        
        # Obtener valores de impedancia - usar data_provider si está disponible
        if data_provider:
            _, resistance_values = data_provider()
        else:
            # Fallback al método directo si no hay data_provider
            resistance_values = get_resistance_values(board, electrodes)
            
        # Verificar si todos los electrodos tienen buena impedancia
        all_ok = all(get_impedance_status(resistance_values[e])[1] <= 2 for e in electrodes)
        
        # Mensaje según el estado de los electrodos
        if all_ok:
            status_msg = "¡Todos los electrodos tienen buen contacto! Presione ENTER para continuar"
        else:
            status_msg = "Ajuste los electrodos hasta que todos tengan contacto EXCELENTE o ACEPTABLE"
        
        # Dibujar gráficos de impedancia para cada electrodo
        need_sound = False
        for electrode in electrodes:
            value = resistance_values[electrode]
            x, y = positions[electrode]
            
            level = view_electrode_graph(
                term, x, y, graph_width, graph_height,
                electrode, value, history, APP_STATE_SETUP
            )
            
            # Para electrodos con mal contacto (nivel > 2), emitir alerta cada 10 actualizaciones
            if level > 2 and update_count % 10 == 0:
                need_sound = True
                
        # Emitir alerta de sonido si es necesario (una sola vez por renderizado)
        if need_sound:
            play_sound("Mal contacto")
        
        # Mostrar barra de estado
        view_status_bar(term, height - 5, APP_STATE_SETUP, status_msg)
        
        # Comprobar input del usuario
        key = term.inkey(timeout=0.1, esc_delay=0)

        if key:
            # Imprimir información detallada sobre la tecla presionada
            key_info = f"Tecla: {repr(key)}, "
            if hasattr(key, 'code'):
                key_info += f"Código: {key.code}, "
            if hasattr(key, 'name'):
                key_info += f"Nombre: {key.name}, "
            key_info += f"All OK: {all_ok}"
            
            print(term.move_xy(2, height - 3) + term.clear_eol + key_info)
            
            # Detectar ENTER de múltiples formas posibles
            is_enter = (key == '\n' or key == '\r' or 
                       (hasattr(key, 'name') and key.name == 'KEY_ENTER') or
                       (hasattr(key, 'code') and key.code == 13))
            
            # ENTER: Continuar solo si la impedancia es correcta
            if is_enter:
                if all_ok:
                    print(term.move_xy(2, height - 2) + term.clear_eol + "¡ENTER detectado con buena impedancia!")
                    play_sound("Impedancia correcta, continuar")
                    return {'event': 'enter_pressed', 'impedance_ok': True}
                else:
                    print(term.move_xy(2, height - 2) + term.clear_eol + "ENTER detectado pero impedancia NO es correcta")
                    play_sound("Impedancia incorrecta")
            
            # ESC o Q: Cancelar en cualquier momento
            is_escape = (key == 'q' or key == 'Q' or
                        (hasattr(key, 'code') and key.code == term.KEY_ESCAPE))
            
            if is_escape:
                print(term.move_xy(2, height - 2) + term.clear_eol + "ESC/Q detectado - Cancelando")
                play_sound("Operación cancelada")
                return {'event': 'cancel'}
        
    except Exception as e:
        print(term.move_xy(1, 1) + term.clear + term.bold_red(f"Error: {str(e)}"))
        view_status_bar(term, height - 5, APP_STATE_ERROR, f"Error al obtener datos de impedancia: {str(e)}")
        time.sleep(3)
        return False
        
    # Devolver None si no hay eventos especiales
    return None