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
Visualization and layout functions for the EEG application terminal interface.
This module provides the base functions to build the user interface
in different application states.
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
    """Draws the interface header using blessed"""
    title = "EEG CAPTURE SYSTEM - BRAINBIT"
    if scenario_type:
        title += f" - SCENARIO {scenario_type.upper()}"
        
    print(term.move_y(y_start) + term.center(term.bold(title), term.width))
    print(term.move_y(y_start + 2) + term.center("=" * (term.width - 4), term.width))
 
def view_electrode_graph(term, x_start, y_start, width, height, electrode, value, history, app_state=APP_STATE_SETUP, eeg_value=None):
    """
    Draws a line graph of impedance or EEG data according to the state.
    
    Args:
        term: Blessed terminal
        x_start, y_start: Initial position for drawing
        width, height: Graph dimensions
        electrode: Electrode name (T3, T4, O1, O2)
        value: Current impedance value
        history: Dictionary with value history
        app_state: Current application state
        eeg_value: Current EEG value (only for non-SETUP states)
    """
    # Determine if we show impedance or EEG data
    show_impedance = (app_state == APP_STATE_SETUP)
    
    if show_impedance:
        # Impedance processing (kOhm)
        status, level = get_impedance_status(value)
        color_func = get_color_for_level(term, level)
        symbol, message = get_symbol_and_message(level)
        
        # Add current value to impedance history
        if f"{electrode}_imp" not in history:
            history[f"{electrode}_imp"] = deque(maxlen=MAX_HISTORY)
        history[f"{electrode}_imp"].append(value)
        
        display_value = value
        unit = "kΩ"
        current_history = history[f"{electrode}_imp"]
    else:
        # EEG data processing (μV)
        if eeg_value is None:
            eeg_value = 0
            
        # Add current value to EEG history
        if f"{electrode}_eeg" not in history:
            history[f"{electrode}_eeg"] = deque(maxlen=MAX_HISTORY)
        history[f"{electrode}_eeg"].append(eeg_value)
        
        display_value = eeg_value
        unit = "μV"
        current_history = history[f"{electrode}_eeg"]
        level = 1  # No levels for EEG, use 1 as default for green color
    
    # Draw frame with more padding
    print(term.move_xy(x_start, y_start) + "┌" + "─" * (width - 2) + "┐")
    for i in range(height - 2):
        print(term.move_xy(x_start, y_start + i + 1) + "│" + " " * (width - 2) + "│")
    print(term.move_xy(x_start, y_start + height - 1) + "└" + "─" * (width - 2) + "┘")
    
    # Show label and current value with appropriate units
    print(term.move_xy(x_start + 3, y_start) + term.bold(f" {electrode} "))
    
    if show_impedance:
        color_func = get_color_for_level(term, level)
        print(term.move_xy(x_start + 3, y_start + 1) + term.clear_eol + 
            f"{display_value:.1f} {unit} {color_func(term.bold(f'{symbol}'))} {message}")
    else:
        print(term.move_xy(x_start + 3, y_start + 1) + term.clear_eol + 
            f"{display_value:.1f} {unit}")
    
    # Configure graph dimensions with more padding
    graph_width = width - 8    # More lateral padding
    graph_height = height - 5  # More vertical padding
    graph_x = x_start + 4      # Greater padding from left edge
    graph_y = y_start + 3      # Greater padding from top edge
    
    # Calculate min and max values to scale the graph
    if current_history:
        if show_impedance:
            max_val = max(max(current_history), IMPEDANCE_POOR * 1.1)
            min_val = max(0, min(current_history) * 0.9)
        else:
            # For EEG, adjust dynamic scale
            max_val = max(max(current_history) * 1.1, 100)  # Minimum 100 μV range
            min_val = min(current_history) * 0.9
    else:
        if show_impedance:
            max_val = IMPEDANCE_POOR * 1.1
            min_val = 0
        else:
            max_val = 100   # 100 μV by default
            min_val = -100  # -100 μV by default
    
    value_range = max(max_val - min_val, 1)  # Avoid division by zero
    
    # Draw value labels with appropriate unit
    print(term.move_xy(graph_x, graph_y + graph_height) + f"{int(min_val)} {unit}")
    print(term.move_xy(graph_x + graph_width - len(str(int(max_val))) - len(unit) - 1, 
                      graph_y + graph_height) + f"{int(max_val)} {unit}")
    
    # Draw threshold lines only for impedance
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
    
    # Draw graph line
    for i, val in enumerate(current_history):
        if i >= graph_width:
            break
        
        # Normalize value to graph height range
        try:
            normalized = (val - min_val) / value_range
        except ZeroDivisionError:
            normalized = 0.5  # Default center value
            
        y_pos = int(graph_y + graph_height - normalized * graph_height)
        
        # Determine color
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
            # For EEG, a color scale according to amplitude
            point_color = term.green
        
        # Draw point at the top
        if graph_y <= y_pos < graph_y + graph_height:
            # Draw the main point
            print(term.move_xy(graph_x + i, y_pos) + point_color('•'))
            
            # Fill the bottom part with more subtle characters
            for fill_y in range(y_pos + 1, graph_y + graph_height):
                print(term.move_xy(graph_x + i, fill_y) + point_color('│'))
    
    return level if show_impedance else 1

def view_status_bar(term, y_pos, app_state, message="", countdown=None, progress=None):
    """
    Shows the status bar at the bottom.
    
    Args:
        term: Blessed terminal
        y_pos: Vertical position for the status bar
        app_state: Current application state
        message: Message to display
        countdown: Tuple (current_time, total_time) to show countdown
        progress: Tuple (current_progress, total_progress) to show progress bar
    """
    width = term.width
    
    # Show dividing line
    print(term.move_xy(1, y_pos) + "=" * (width - 2))
    
    # Show current state
    state_messages = {
        APP_STATE_SETUP: "ADJUSTING HEADSET",
        APP_STATE_COUNTDOWN: "PREPARING CAPTURE",
        APP_STATE_CAPTURE: "CAPTURING DATA",
        APP_STATE_COMPLETE: "CAPTURE COMPLETED",
        APP_STATE_ERROR: "ERROR"
    }
    
    state_colors = {
        APP_STATE_SETUP: term.yellow,
        APP_STATE_COUNTDOWN: term.blue,
        APP_STATE_CAPTURE: term.green,
        APP_STATE_COMPLETE: term.cyan,
        APP_STATE_ERROR: term.red
    }
    
    state_text = state_messages.get(app_state, "UNKNOWN")
    color_func = state_colors.get(app_state, lambda x: x)
    
    print(term.move_xy(2, y_pos + 1) + term.clear_eol + 
          f"State: {color_func(term.bold(state_text))} - {message}")
    
    # Show progress bar if needed
    if countdown is not None:
        bar_width = 20
        filled = int((countdown[0] / countdown[1]) * bar_width)
        bar = '■' * filled + '░' * (bar_width - filled)
        print(term.move_xy(width - bar_width - 15, y_pos + 1) + 
              f"Time: [{bar}] {countdown[0]}s")
    
    elif progress is not None:
        bar_width = 20
        filled = int((progress[0] / progress[1]) * bar_width)
        bar = '■' * filled + '░' * (bar_width - filled)
        print(term.move_xy(width - bar_width - 15, y_pos + 1) + 
              f"Progress: [{bar}] {progress[0]}/{progress[1]}")
    
    # Show instructions
    if app_state == APP_STATE_SETUP:
        print(term.move_xy(2, y_pos + 2) + term.clear_eol + 
              "Press ENTER when ready to start or ESC to cancel")
    else:
        print(term.move_xy(2, y_pos + 2) + term.clear_eol + 
              "ESC to cancel capture")