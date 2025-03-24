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
"""
Configuration for the EEG capture system.
"""

# Capture parameters
NUM_SAMPLES = 100              # Total number of samples to capture
WINDOW_SECONDS = 10            # Duration of each capture window (seconds)
INITIAL_DELAY = 30             # Initial delay before starting capture (seconds)
BEEP_FREQ = 1000               # Alert sound frequency (Hz)
MAX_HISTORY = 50               # Maximum number of points in the graph history

# BrainBit device configuration
BRAINBIT_MAC_DEFAULT = "C8:8F:B6:6D:E1:E2"  # Default MAC address

# EEG channel mapping
CHANNEL_MAP = {
    "timestamp": 10,
    "T3": 1,
    "T4": 2,
    "O1": 3,
    "O2": 4
}

# Impedance channel mapping
IMPEDANCE_MAP = {
    "T3": 5,
    "T4": 6,
    "O1": 7,
    "O2": 8
}

# Impedance limits (kΩ)
IMPEDANCE_TOO_LOW = 200      # Minimum acceptable impedance (kΩ)
IMPEDANCE_EXCELLENT = 800    # Optimal contact impedance (kΩ)
IMPEDANCE_ACCEPTABLE = 1500  # Acceptable impedance (kΩ)
IMPEDANCE_POOR = 2000        # Poor connection impedance (kΩ)

# Application states
APP_STATE_SETUP = "setup"          # Headset configuration/initialization
APP_STATE_COUNTDOWN = "countdown"  # Countdown before capture
APP_STATE_CAPTURE = "capture"      # EEG data capture
APP_STATE_COMPLETE = "complete"    # Capture completed
APP_STATE_ERROR = "error"          # Capture error

if __name__ == "__main__":
    # Simple configuration test
    print("EEG capture system configuration:")
    print(f"NUM_SAMPLES = {NUM_SAMPLES}")
    print(f"WINDOW_SECONDS = {WINDOW_SECONDS}")
    print(f"INITIAL_DELAY = {INITIAL_DELAY}")
    print(f"BRAINBIT_MAC_DEFAULT = {BRAINBIT_MAC_DEFAULT}")
