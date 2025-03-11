import math
import numpy as np
from brainflow.board_shim import BoardShim, BoardIds, BrainFlowInputParams


from config.settings import BRAINBIT_MAC_DEFAULT, IMPEDANCE_MAP
from processing.impedance import ohm_to_kohm

def configure_board(mac_address=None):
    """
    Inicializa y configura la conexión con el dispositivo BrainBit.
    """
    params = BrainFlowInputParams()
    params.mac_address = mac_address or BRAINBIT_MAC_DEFAULT
    params.timeout = 20
    
    board = BoardShim(BoardIds.BRAINBIT_BOARD, params)
    return board

def get_resistance_values(board, electrodes):
    """
    Obtiene las medidas de resistencia de los electrodos del BrainBit.
    NOTA: Esta función asume que el dispositivo ya está en modo de impedancia.
    """
    try:
        # Obtener datos de impedancia (asumiendo que ya estamos en modo impedancia)
        data = board.get_board_data(100)
        
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
                        values[electrode] = 3000 # Valor por defecto en kOhm
                else:
                    values[electrode] = 3000  # Valor por defecto en kOhm
        else:
            values = {electrode: 3000 for electrode in electrodes}
        
        return values
    except Exception as e:
        print(f"Error obteniendo resistencias: {str(e)}")
        return {electrode: 3300 for electrode in electrodes}
