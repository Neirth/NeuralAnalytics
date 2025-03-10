"""
Módulo para el procesamiento de datos de impedancia.

Este módulo proporciona funciones para el procesamiento de datos de impedancia
obtenidos de un dispositivo EEG.
"""
def ohm_to_kohm(ohm_value):
    """
    Convierte valores de Ohm a kOhm.
    
    Args:
        ohm_value: Valor en Ohm
        
    Returns:
        float: Valor en kOhm
    """
    return ohm_value / 1000.0