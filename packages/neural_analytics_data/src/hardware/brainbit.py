# Copyright (C) 2025 Sergio Martínez Aznar
# 
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
# 
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
# 
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.
import numpy as np
from brainflow.board_shim import BoardShim, BoardIds, BrainFlowInputParams


from config.settings import BRAINBIT_MAC_DEFAULT, IMPEDANCE_MAP
from processing.impedance import ohm_to_kohm

def configure_board(mac_address=None):
    """
    Initializes and configures the connection with the BrainBit device.
    """
    params = BrainFlowInputParams()
    params.mac_address = mac_address or BRAINBIT_MAC_DEFAULT
    params.timeout = 20
    
    board = BoardShim(BoardIds.BRAINBIT_BOARD, params)
    return board

def get_resistance_values(board, electrodes):
    """
    Gets the resistance measurements from the BrainBit electrodes.
    NOTE: This function assumes that the device is already in impedance mode.
    """
    try:
        # Get impedance data (assuming we are already in impedance mode)
        data = board.get_board_data(100)
        
        values = {}
        if data.size > 0:
            for electrode in electrodes:
                imp_channel = IMPEDANCE_MAP[electrode]
                if imp_channel < data.shape[0]:
                    # Get absolute resistance values in Ohm
                    resistance_values_ohm = np.abs(data[imp_channel])
                    
                    # Convert to kOhm for filtering and processing
                    resistance_values_kohm = ohm_to_kohm(resistance_values_ohm)
                    
                    # Filter valid values (now in kOhm)
                    valid_values = resistance_values_kohm[(resistance_values_kohm > 1) & (resistance_values_kohm < 5000)]
                    
                    if len(valid_values) > 0:
                        values[electrode] = np.mean(valid_values)
                    else:
                        values[electrode] = 3000 # Default value in kOhm
                else:
                    values[electrode] = 3000  # Default value in kOhm
        else:
            values = {electrode: 3000 for electrode in electrodes}
        
        return values
    except Exception as e:
        print(f"Error getting resistances: {str(e)}")
        return {electrode: 3300 for electrode in electrodes}
