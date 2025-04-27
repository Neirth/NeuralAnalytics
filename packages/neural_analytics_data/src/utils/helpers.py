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
import os

# Import configuration
from config.settings import (
    IMPEDANCE_TOO_LOW,
    IMPEDANCE_EXCELLENT,
    IMPEDANCE_ACCEPTABLE,
    IMPEDANCE_POOR
)

def play_sound(message=""):
    """Play system beep using macOS tools"""
    os.system(f'say "{message}"')
    # pass

def get_impedance_status(value):
    """Returns status based on impedance value"""
    if value < IMPEDANCE_TOO_LOW:
        return "CRITICAL LOW", 5  # Red (contact too strong/short circuit)
    elif value < IMPEDANCE_EXCELLENT:
        return "EXCELLENT", 1  # Green
    elif value < IMPEDANCE_ACCEPTABLE:
        return "ACCEPTABLE", 2  # Yellow
    elif value < IMPEDANCE_POOR:
        return "CHECK", 3  # Orange
    else:
        return "CRITICAL HIGH", 4  # Red (poor contact)

def get_color_for_level(term, level):
    """Returns the appropriate color based on impedance level"""
    if level == 1:
        return term.green
    elif level == 2:
        return term.yellow
    elif level == 3:
        return term.magenta
    else:  # level == 4 or level == 5
        return term.red

def get_symbol_and_message(level):
    """Returns symbol and message based on impedance level"""
    if level == 1:
        return "✓", "Optimal contact"
    elif level == 2:
        return "⚠", "Acceptable contact"
    elif level == 3:
        return "!", "Adjust position"
    elif level == 4:
        return "✗", "Poor contact, reposition"
    else:  # level == 5
        return "✗", "Contact too strong or short-circuit"