import os

# Importar configuración
from config.settings import (
    IMPEDANCIA_TOO_LOW,
    IMPEDANCE_EXCELLENT,
    IMPEDANCE_ACCEPTABLE,
    IMPEDANCE_POOR
)

def play_sound(message=""):
    """Play system beep using macOS tools"""
    #os.system(f'say "{message}"')
    pass

def get_impedance_status(value):
    """Devuelve el estado según el valor de impedancia"""
    if value < IMPEDANCIA_TOO_LOW:
        return "CRÍTICO BAJO", 5  # Rojo (contacto demasiado fuerte/cortocircuito)
    elif value < IMPEDANCE_EXCELLENT:
        return "EXCELENTE", 1  # Verde
    elif value < IMPEDANCE_ACCEPTABLE:
        return "ACEPTABLE", 2  # Amarillo
    elif value < IMPEDANCE_POOR:
        return "REVISAR", 3  # Naranja
    else:
        return "CRÍTICO ALTO", 4  # Rojo (mal contacto)

def get_color_for_level(term, level):
    """Devuelve el color apropiado según el nivel de impedancia"""
    if level == 1:
        return term.green
    elif level == 2:
        return term.yellow
    elif level == 3:
        return term.magenta
    else:  # level == 4 o level == 5
        return term.red

def get_symbol_and_message(level):
    """Devuelve el símbolo y mensaje según el nivel de impedancia"""
    if level == 1:
        return "✓", "Contacto óptimo"
    elif level == 2:
        return "⚠", "Contacto aceptable"
    elif level == 3:
        return "!", "Ajustar posición"
    elif level == 4:
        return "✗", "Mal contacto, recolocar"
    else:  # level == 5
        return "✗", "Contacto demasiado fuerte o cortocircuito"