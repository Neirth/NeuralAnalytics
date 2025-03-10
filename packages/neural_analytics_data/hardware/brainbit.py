import numpy as np
from brainflow.board_shim import BoardShim, BoardIds, BrainFlowInputParams


from config.settings import BRAINBIT_MAC_DEFAULT, IMPEDANCE_MAP

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
                    resistance_values = np.abs(data[imp_channel])
                    valid_values = resistance_values[(resistance_values > 1) & (resistance_values < 5000)]
                    if len(valid_values) > 0:
                        values[electrode] = np.mean(valid_values)
                    else:
                        values[electrode] = 2000
                else:
                    values[electrode] = 2000
        else:
            values = {electrode: 2000 for electrode in electrodes}
        
        return values
    except Exception as e:
        print(f"Error obteniendo resistencias: {str(e)}")
        return {electrode: 2000 for electrode in electrodes}
