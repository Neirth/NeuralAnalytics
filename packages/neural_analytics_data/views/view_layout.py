"""
Funciones de visualización y layout para la interfaz terminal de la aplicación EEG.
Este módulo proporciona las funciones base para construir la interfaz de usuario
en diferentes estados de la aplicación.
"""

from blessed import Terminal
from collections import deque

from config.settings import (
    APP_STATE_SETUP, APP_STATE_COUNTDOWN, APP_STATE_CAPTURE, 
    APP_STATE_COMPLETE, APP_STATE_ERROR,
    IMPEDANCE_EXCELLENT, IMPEDANCE_ACCEPTABLE, IMPEDANCE_POOR, MAX_HISTORY
)

from utils.helpers import get_impedance_status, get_color_for_level, get_symbol_and_message

def view_header(term, y_start=1, scenario_type=None):
    """Dibuja el encabezado de la interfaz usando blessed"""
    title = "SISTEMA DE CAPTURA EEG - BRAINBIT"
    if scenario_type:
        title += f" - ESCENARIO {scenario_type.upper()}"
        
    print(term.move_y(y_start) + term.center(term.bold(title), term.width))
    print(term.move_y(y_start + 2) + term.center("=" * (term.width - 4), term.width))
 
def view_electrode_graph(term, x_start, y_start, width, height, electrode, value, history, app_state=APP_STATE_SETUP, eeg_value=None):
    """
    Dibuja una gráfica lineal de impedancia o datos EEG según el estado.
    
    Args:
        term: Terminal blessed
        x_start, y_start: Posición inicial para dibujar
        width, height: Dimensiones de la gráfica
        electrode: Nombre del electrodo (T3, T4, O1, O2)
        value: Valor de impedancia actual
        history: Diccionario con históricos de valores
        app_state: Estado actual de la aplicación
        eeg_value: Valor EEG actual (solo para estados no SETUP)
    """
    # Determinar si mostramos impedancia o datos EEG
    show_impedance = (app_state == APP_STATE_SETUP)
    
    if show_impedance:
        # Procesamiento de impedancia (kOhm)
        status, level = get_impedance_status(value)
        color_func = get_color_for_level(term, level)
        symbol, message = get_symbol_and_message(level)
        
        # Añadir valor actual al historial de impedancia
        if f"{electrode}_imp" not in history:
            history[f"{electrode}_imp"] = deque(maxlen=MAX_HISTORY)
        history[f"{electrode}_imp"].append(value)
        
        display_value = value
        unit = "kΩ"
        current_history = history[f"{electrode}_imp"]
    else:
        # Procesamiento de datos EEG (μV)
        if eeg_value is None:
            eeg_value = 0
            
        # Añadir valor actual al historial EEG
        if f"{electrode}_eeg" not in history:
            history[f"{electrode}_eeg"] = deque(maxlen=MAX_HISTORY)
        history[f"{electrode}_eeg"].append(eeg_value)
        
        display_value = eeg_value
        unit = "μV"
        current_history = history[f"{electrode}_eeg"]
        level = 1  # No hay niveles para EEG, usar 1 como predeterminado para color verde
    
    # Dibujar marco con más padding
    print(term.move_xy(x_start, y_start) + "┌" + "─" * (width - 2) + "┐")
    for i in range(height - 2):
        print(term.move_xy(x_start, y_start + i + 1) + "│" + " " * (width - 2) + "│")
    print(term.move_xy(x_start, y_start + height - 1) + "└" + "─" * (width - 2) + "┘")
    
    # Mostrar etiqueta y valor actual con unidades apropiadas
    print(term.move_xy(x_start + 3, y_start) + term.bold(f" {electrode} "))
    
    if show_impedance:
        color_func = get_color_for_level(term, level)
        print(term.move_xy(x_start + 3, y_start + 1) + term.clear_eol + 
            f"{display_value:.1f} {unit} {color_func(term.bold(f'{symbol}'))} {message}")
    else:
        print(term.move_xy(x_start + 3, y_start + 1) + term.clear_eol + 
            f"{display_value:.1f} {unit}")
    
    # Configurar dimensiones de la gráfica con más padding
    graph_width = width - 8  # Más padding lateral
    graph_height = height - 5  # Más padding vertical
    graph_x = x_start + 4     # Mayor padding desde el borde izquierdo
    graph_y = y_start + 3     # Mayor padding desde el borde superior
    
    # Calcular valores min y max para escalar la gráfica
    if current_history:
        if show_impedance:
            max_val = max(max(current_history), IMPEDANCE_POOR * 1.1)
            min_val = max(0, min(current_history) * 0.9)
        else:
            # Para EEG, ajustar escala dinámica
            max_val = max(max(current_history) * 1.1, 100)  # Mínimo 100 μV de rango
            min_val = min(current_history) * 0.9
    else:
        if show_impedance:
            max_val = IMPEDANCE_POOR * 1.1
            min_val = 0
        else:
            max_val = 100  # 100 μV por defecto
            min_val = -100  # -100 μV por defecto
    
    value_range = max(max_val - min_val, 1)  # Evitar división por cero
    
    # Dibujar etiquetas de valores con unidad apropiada
    print(term.move_xy(graph_x, graph_y + graph_height) + f"{int(min_val)} {unit}")
    print(term.move_xy(graph_x + graph_width - len(str(int(max_val))) - len(unit) - 1, 
                      graph_y + graph_height) + f"{int(max_val)} {unit}")
    
    # Dibujar líneas de umbral solo para impedancia
    if show_impedance:
        poor_pos = int((IMPEDANCE_POOR - min_val) / value_range * graph_width)
        if 0 <= poor_pos < graph_width:
            print(term.move_xy(graph_x + poor_pos, graph_y + graph_height + 1) + term.red(f"P({IMPEDANCE_POOR})"))
        
        acceptable_pos = int((IMPEDANCE_ACCEPTABLE - min_val) / value_range * graph_width)
        if 0 <= acceptable_pos < graph_width:
            print(term.move_xy(graph_x + acceptable_pos, graph_y + graph_height + 1) + term.yellow(f"A({IMPEDANCE_ACCEPTABLE})"))
        
        excellent_pos = int((IMPEDANCE_EXCELLENT - min_val) / value_range * graph_width)
        if 0 <= excellent_pos < graph_width:
            print(term.move_xy(graph_x + excellent_pos, graph_y + graph_height + 1) + term.green(f"E({IMPEDANCE_EXCELLENT})"))
    
    # Dibujar línea de gráfica
    for i, val in enumerate(current_history):
        if i >= graph_width:
            break
        
        # Normalizar valor al rango de altura de la gráfica
        try:
            normalized = (val - min_val) / value_range
        except ZeroDivisionError:
            normalized = 0.5  # Valor central predeterminado
            
        y_pos = int(graph_y + graph_height - normalized * graph_height)
        
        # Determinar color
        if show_impedance:
            if val < IMPEDANCE_EXCELLENT:
                point_color = term.green
            elif val < IMPEDANCE_ACCEPTABLE:
                point_color = term.yellow
            elif val < IMPEDANCE_POOR:
                point_color = term.magenta
            else:
                point_color = term.red
        else:
            # Para EEG, una escala de colores según la amplitud
            point_color = term.green
        
        # Dibujar punto en la parte superior
        if graph_y <= y_pos < graph_y + graph_height:
            # Dibujar el punto principal
            print(term.move_xy(graph_x + i, y_pos) + point_color('•'))
            
            # Rellenar la parte inferior con caracteres más sutiles
            for fill_y in range(y_pos + 1, graph_y + graph_height):
                print(term.move_xy(graph_x + i, fill_y) + point_color('│'))
    
    return level if show_impedance else 1

def view_status_bar(term, y_pos, app_state, message="", countdown=None, progress=None):
    """
    Muestra la barra de estado en la parte inferior.
    
    Args:
        term: Terminal blessed
        y_pos: Posición vertical para la barra de estado
        app_state: Estado actual de la aplicación
        message: Mensaje a mostrar
        countdown: Tupla (tiempo_actual, tiempo_total) para mostrar cuenta regresiva
        progress: Tupla (progreso_actual, progreso_total) para mostrar barra de progreso
    """
    width = term.width
    
    # Mostrar línea divisoria
    print(term.move_xy(1, y_pos) + "=" * (width - 2))
    
    # Mostrar estado actual
    state_messages = {
        APP_STATE_SETUP: "AJUSTANDO CASCO",
        APP_STATE_COUNTDOWN: "PREPARANDO CAPTURA",
        APP_STATE_CAPTURE: "CAPTURANDO DATOS",
        APP_STATE_COMPLETE: "CAPTURA COMPLETADA",
        APP_STATE_ERROR: "ERROR"
    }
    
    state_colors = {
        APP_STATE_SETUP: term.yellow,
        APP_STATE_COUNTDOWN: term.blue,
        APP_STATE_CAPTURE: term.green,
        APP_STATE_COMPLETE: term.cyan,
        APP_STATE_ERROR: term.red
    }
    
    state_text = state_messages.get(app_state, "DESCONOCIDO")
    color_func = state_colors.get(app_state, lambda x: x)
    
    print(term.move_xy(2, y_pos + 1) + term.clear_eol + 
          f"Estado: {color_func(term.bold(state_text))} - {message}")
    
    # Mostrar barra de progreso si es necesario
    if countdown is not None:
        bar_width = 20
        filled = int((countdown[0] / countdown[1]) * bar_width)
        bar = '■' * filled + '░' * (bar_width - filled)
        print(term.move_xy(width - bar_width - 15, y_pos + 1) + 
              f"Tiempo: [{bar}] {countdown[0]}s")
    
    elif progress is not None:
        bar_width = 20
        filled = int((progress[0] / progress[1]) * bar_width)
        bar = '■' * filled + '░' * (bar_width - filled)
        print(term.move_xy(width - bar_width - 15, y_pos + 1) + 
              f"Progreso: [{bar}] {progress[0]}/{progress[1]}")
    
    # Mostrar instrucciones
    if app_state == APP_STATE_SETUP:
        print(term.move_xy(2, y_pos + 2) + term.clear_eol + 
              "Presione ENTER cuando esté listo para comenzar o ESC para cancelar")
    else:
        print(term.move_xy(2, y_pos + 2) + term.clear_eol + 
              "ESC para cancelar la captura")