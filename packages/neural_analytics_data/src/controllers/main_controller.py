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
Main controller for the EEG capture system.
Manages the transition between views, communication with hardware,
and data capture and storage.
"""

import os
import time
import numpy as np
import threading
import signal
from blessed import Terminal

# Import system components
from config.settings import (
    APP_STATE_SETUP, APP_STATE_COUNTDOWN, APP_STATE_CAPTURE, 
    APP_STATE_COMPLETE, APP_STATE_ERROR, IMPEDANCE_MAP,
    CHANNEL_MAP, INITIAL_DELAY, NUM_SAMPLES, WINDOW_SECONDS
)

from hardware.brainbit import configure_board
from processing.impedance import ohm_to_kohm
from views.view_impedance_data import view_impedance_screen
from views.view_capturer_data import view_capture_screen
from utils.helpers import play_sound

class NeuralCaptureController:
    def __init__(self, scenario_type="red", mac_address=None):
        """
        Initializes the neural capture controller.
        
        Args:
            scenario_type: Type of scenario ('red' or 'green')
            mac_address: MAC address of the BrainBit device (optional)
        """
        self.scenario_type = scenario_type
        self.mac_address = mac_address
        self.board = None
        self.app_state = APP_STATE_SETUP
        self.term = Terminal()
        self.current_device_mode = None  # To track the current device mode
        
        # Shared state
        self.history = {}
        self.state_info = {
            'app_state': APP_STATE_SETUP,
            'message': "Initializing...",
            'scenario_type': scenario_type,
            'capture_count': 0,
            'countdown_seconds': INITIAL_DELAY
        }
        
        # Control events for threads
        self.control_event = threading.Event()
        self.data_ready_event = threading.Event()
        
        # Current data (shared between threads)
        self.current_eeg_values = {electrode: 0 for electrode in ["T3", "T4", "O1", "O2"]}
        self.current_imp_values = {electrode: 0 for electrode in ["T3", "T4", "O1", "O2"]}
        self.lock = threading.Lock()  # For safe access to shared data
    
    def initialize_hardware(self):
        """Initializes the connection with BrainBit hardware"""
        try:
            self.board = configure_board(self.mac_address)
            self.board.prepare_session()
            self.board.start_stream(450000)
            # Don't start with any specific mode here
            return True
        except Exception as e:
            self.state_info['message'] = f"Error initializing hardware: {str(e)}"
            self.app_state = APP_STATE_ERROR
            return False
    
    def set_device_mode(self, mode):
        """
        Configures the BrainBit device mode.
        
        Args:
            mode: 'signal' for EEG mode, 'impedance' for impedance mode
        """
        if not self.board:
            return False
        
        # If we're already in the requested mode, do nothing
        if self.current_device_mode == mode:
            return True
            
        try:
            # Stop the current mode if it exists
            if self.current_device_mode == 'signal':
                self.board.config_board('CommandStopSignal')
            elif self.current_device_mode == 'impedance':
                self.board.config_board('CommandStopResist')
                
            # Activate the new mode
            if mode == 'signal':
                self.board.config_board('CommandStartSignal')
                self.current_device_mode = 'signal'
            elif mode == 'impedance':
                self.board.config_board('CommandStartResist')
                self.current_device_mode = 'impedance'
                
            # Small delay for stabilization
            time.sleep(1)
            return True
        except Exception as e:
            print(f"Error changing device mode: {str(e)}")
            return False
    
    def cleanup_hardware(self):
        """Cleans up and closes the hardware connection"""
        if self.board:
            try:
                # Stop the active mode before releasing
                if self.current_device_mode == 'signal':
                    self.board.config_board('CommandStopSignal')
                elif self.current_device_mode == 'impedance':
                    self.board.config_board('CommandStopResist')
                    
                self.board.stop_stream()
                self.board.release_session()
            except Exception as e:
                print(f"Error releasing session: {str(e)}")
    
    def data_provider(self):
        """
        Data provider for views.
        Returns current EEG and impedance values.
        """
        with self.lock:
            return self.current_eeg_values.copy(), self.current_imp_values.copy()
    
    def update_data_thread(self):
        """
        Thread to update device data (EEG and impedance).
        Runs in the background and updates current values.
        """
        electrodes = ["T3", "T4", "O1", "O2"]
        last_app_state = None
        
        while not self.control_event.is_set():
            try:
                # Verify that the board exists and is ready
                if self.board is None:
                    time.sleep(0.5)
                    continue
                    
                # Check if the application state has changed
                if last_app_state != self.app_state:
                    # Change device mode according to the new state
                    if self.app_state == APP_STATE_SETUP:
                        self.set_device_mode('impedance')
                    else:  # APP_STATE_COUNTDOWN, APP_STATE_CAPTURE, etc.
                        self.set_device_mode('signal')
                    
                    last_app_state = self.app_state
                    # Give additional time for the device to stabilize
                    time.sleep(1.0)
                
                # Get data according to current mode
                if self.current_device_mode == 'impedance':
                    try:
                        # In impedance mode, get values directly without changing the mode
                        data = self.board.get_board_data(100)
                        
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
                                        values[electrode] = 4000  # Default value in kOhm
                                else:
                                    values[electrode] = 4000  # Default value in kOhm
                        else:
                            values = {electrode: 4000 for electrode in electrodes}
                        
                        with self.lock:
                            self.current_imp_values = values
                        
                    except Exception as e:
                        print(f"Error getting impedance data: {str(e)}")
                        time.sleep(0.5)  # Wait before retrying
                        continue  # Continue with the next cycle of the loop
                
                elif self.current_device_mode == 'signal':
                    try:
                        # In signal mode, get EEG values
                        data = self.board.get_board_data(100)
                        
                        # Process EEG data for visualization
                        if data.size > 0:
                            eeg_values = {}
                            for electrode in electrodes:
                                eeg_channel = CHANNEL_MAP[electrode]
                                if eeg_channel < data.shape[0]:
                                    eeg_data = data[eeg_channel]
                                    eeg_values[electrode] = np.mean(np.abs(eeg_data))
                                else:
                                    eeg_values[electrode] = 0
                            
                            with self.lock:
                                self.current_eeg_values = eeg_values
                    
                    except Exception as e:
                        print(f"Error getting EEG data: {str(e)}")
                        time.sleep(0.5)
                        continue
                
                # Indicate that new data is available
                self.data_ready_event.set()
                self.data_ready_event.clear()
                
                # Wait a bit to avoid overloading the device
                time.sleep(0.2)
                
            except Exception as e:
                print(f"Error in data thread: {str(e)}")
                time.sleep(1)  # Wait before retrying
    
    def capture_data_thread(self):
        """
        Thread to capture EEG data at the right time.
        Only runs when we are in capture state.
        """
        last_capture_time = 0
        capture_count = 0
        buffer_data = None  # Accumulated data buffer
        
        while not self.control_event.is_set():
            try:
                # Only capture when we are in capture state
                if self.app_state != APP_STATE_CAPTURE:
                    time.sleep(0.5)
                    continue
                    
                current_time = time.time()
                
                # Update counter visible to the user
                with self.lock:
                    self.state_info['capture_count'] = capture_count
                
                # Only capture a segment every WINDOW_SECONDS
                if current_time - last_capture_time >= WINDOW_SECONDS:
                    try:
                        # Get more complete EEG data for saving
                        data = self.board.get_board_data(250 * WINDOW_SECONDS)
                        
                        # Select only relevant channels
                        selected_data = np.take(data, [
                            CHANNEL_MAP["timestamp"] if "timestamp" in CHANNEL_MAP else 10,
                            CHANNEL_MAP["T3"],
                            CHANNEL_MAP["T4"], 
                            CHANNEL_MAP["O1"],
                            CHANNEL_MAP["O2"]
                        ], axis=0).T
                        
                        # Add data to buffer
                        if buffer_data is None:
                            buffer_data = selected_data
                        else:
                            buffer_data = np.vstack((buffer_data, selected_data))
                        
                        print(f"[INFO] Capture {capture_count+1}: Obtained {len(selected_data)} rows. Accumulated buffer: {len(buffer_data)} rows")
                        
                        # While we have enough data, save files of 100 rows
                        while len(buffer_data) >= 100:
                            # Prepare directory for saving
                            base_dir = f"../dataset/{self.scenario_type}/"
                            os.makedirs(base_dir, exist_ok=True)
                            timestamp = time.strftime("%Y%m%d-%H%M%S")
                            filename = f"{base_dir}{self.scenario_type}_{timestamp}_{capture_count:03d}.csv"
                            
                            # Extract exactly 100 rows
                            save_data = buffer_data[:100]
                            # Keep the rest in the buffer
                            buffer_data = buffer_data[100:]
                            
                            # Save to CSV
                            np.savetxt(filename, save_data, delimiter=',',
                                      header="timestamp,T3,T4,O1,O2",
                                      fmt=['%.3f', '%.6f', '%.6f', '%.6f', '%.6f'])
                            
                            print(f"[INFO] CSV file saved: {os.path.basename(filename)} with exactly 100 rows")
                            
                            # Increment counter and update message
                            capture_count += 1
                            with self.lock:
                                self.state_info['capture_count'] = capture_count
                                self.state_info['message'] = f"Sample {capture_count}/{NUM_SAMPLES} captured"
                            
                            # Check if we've finished
                            if capture_count >= NUM_SAMPLES:
                                break
                        
                        # Update state
                        last_capture_time = current_time
                        
                        # If we've completed all captures
                        if capture_count >= NUM_SAMPLES:
                            # If there's data left in the buffer and less than 100 rows, fill with zeros
                            if buffer_data is not None and len(buffer_data) > 0:
                                print(f"[INFO] {len(buffer_data)} rows remaining in buffer, discarding...")
                            
                            with self.lock:
                                self.app_state = APP_STATE_COMPLETE
                                self.state_info['app_state'] = APP_STATE_COMPLETE
                                self.state_info['message'] = "Capture completed! Processing data..."
                            break
                            
                    except Exception as e:
                        error_msg = f"Error capturing data: {str(e)}"
                        print(f"[ERROR] {error_msg}")
                        
                        # Only change to error if it's a serious problem
                        if "Fatal" in str(e) or capture_count == 0:  # If it's a fatal error or we haven't captured anything
                            with self.lock:
                                self.app_state = APP_STATE_ERROR
                                self.state_info['app_state'] = APP_STATE_ERROR
                                self.state_info['message'] = error_msg
                            break
                        else:
                            # If we already have some captures, keep trying
                            time.sleep(1)
                
                time.sleep(0.1)
                
            except Exception as e:
                print(f"[ERROR CAPTURE] General error in capture thread: {str(e)}")
                time.sleep(0.5)
        
        # Ensure the result is recorded, whether success or failure
        if capture_count >= NUM_SAMPLES and not self.control_event.is_set():
            print(f"[INFO] Capture completed successfully. {capture_count} CSV files of 100 rows each were saved.")
        else:
            print(f"[INFO] Capture interrupted or incomplete. Only {capture_count}/{NUM_SAMPLES} files were saved.")
    
    def countdown_thread(self):
        """
        Thread to manage the countdown before capture.
        """
        start_time = time.time()
        
        while not self.control_event.is_set():
            if self.app_state != APP_STATE_COUNTDOWN:
                time.sleep(0.5)
                continue
                
            # Calculate remaining time
            elapsed = time.time() - start_time
            countdown_seconds = max(0, INITIAL_DELAY - int(elapsed))
            
            with self.lock:
                self.state_info['countdown_seconds'] = countdown_seconds
            
            # If the countdown reached 0, change to capture mode
            if countdown_seconds <= 0:
                with self.lock:
                    self.app_state = APP_STATE_CAPTURE
                    self.state_info['app_state'] = APP_STATE_CAPTURE
                    self.state_info['message'] = "Capture started. Maintain the indicated position."
                
                play_sound("Capture begins")
                break
                
            time.sleep(0.1)
    
    def handle_keyboard_input(self):
        """
        Handles keyboard input to control the application.
        """
        key = self.term.inkey(timeout=0.1)
        
        if key:
            # Check in multiple ways if it's ENTER (could be '\n', '\r' or KEY_ENTER)
            if key.code == self.term.KEY_ENTER or key == '\n' or key == '\r':
                # Advance to next state according to current state
                if self.app_state == APP_STATE_SETUP:
                    self.app_state = APP_STATE_COUNTDOWN
                    self.state_info['app_state'] = APP_STATE_COUNTDOWN
                    self.state_info['message'] = f"Preparing capture. Acquisition will start in {INITIAL_DELAY} seconds."
                    return {'action': 'next'}
                    
            elif key.code == self.term.KEY_ESCAPE or key == 'q':
                # Cancel operation
                self.control_event.set()
                return {'action': 'cancel'}
            
            # Add debug print to see what key is being pressed
            print(f"Key pressed: {repr(key)}, code: {key.code if hasattr(key, 'code') else 'no code'}")
                
        return {'action': 'none'}
    
    def handle_view_event(self, event_data):
        """
        Processes events generated by views.
        
        Args:
            event_data: Dictionary with event information
            
        Returns:
            dict: Result of event processing
        """
        if not event_data or 'event' not in event_data:
            return {'action': 'none'}
        
        # Handle keyboard events
        if event_data['event'] == 'key_press':
            key = event_data.get('key', '')
            
            if key == 'enter':
                # If it's the impedance screen and impedance is correct
                if self.app_state == APP_STATE_SETUP and event_data.get('impedance_ok', False):
                    self.app_state = APP_STATE_COUNTDOWN
                    self.state_info['app_state'] = APP_STATE_COUNTDOWN
                    self.state_info['message'] = f"Preparing capture. Acquisition will start in {INITIAL_DELAY} seconds."
                    return {'action': 'next'}
                    
            elif key == 'escape':
                # Cancel operation
                self.control_event.set()
                return {'action': 'cancel'}
        
        return {'action': 'none'}
    
    def run(self):
        """
        Main method to run the neural capture flow.
        Manages transitions between states and views.
        """
        # Initialize hardware
        if not self.initialize_hardware():
            print("[ERROR] Error initializing hardware")
            return False
            
        print("[INFO] Hardware initialized correctly")
        
        # Signals to capture Ctrl+C and other events
        def signal_handler(sig, frame):
            print("[INFO] Interruption signal received")
            self.control_event.set()
        
        signal.signal(signal.SIGINT, signal_handler)
        signal.signal(signal.SIGTERM, signal_handler)
        
        try:
            # Start data update thread in the background
            data_thread = threading.Thread(target=self.update_data_thread)
            data_thread.daemon = True
            data_thread.start()
            print("[INFO] Data update thread started")
            
            # 1. Impedance configuration phase
            with self.term.fullscreen(), self.term.hidden_cursor(), self.term.cbreak():
                while not self.control_event.is_set() and self.app_state == APP_STATE_SETUP:
                    # Show impedance view
                    event_result = view_impedance_screen(
                        self.term, self.board, self.history, self.control_event, self.data_provider)
                    
                    # Process view event
                    if isinstance(event_result, dict) and 'event' in event_result:
                        result = self.handle_view_event(event_result)
                        
                        if result['action'] == 'next':
                            print("[INFO] Advancing to countdown phase")
                            break
                        elif result['action'] == 'cancel':
                            play_sound("Operation canceled")
                            print("[INFO] Operation canceled by user")
                            return False
                            
                    elif event_result is False:  # If the view returned False (error)
                        print("[ERROR] Error in impedance screen")
                        return False
                        
                    time.sleep(0.1)
                
                if self.control_event.is_set():
                    print("[INFO] Operation canceled (control event)")
                    return False
                
                # 2. Transition to countdown phase
                self.app_state = APP_STATE_COUNTDOWN
                self.state_info['app_state'] = APP_STATE_COUNTDOWN
                self.state_info['message'] = f"Preparing capture. Acquisition will start in {INITIAL_DELAY} seconds."
                print("[INFO] Starting countdown")
                
                # Start countdown thread
                countdown_thread = threading.Thread(target=self.countdown_thread)
                countdown_thread.daemon = True
                countdown_thread.start()
                
                # Start data capture thread
                capture_thread = threading.Thread(target=self.capture_data_thread)
                capture_thread.daemon = True
                capture_thread.start()
                print("[INFO] Countdown and capture threads started")
                
                # 3. Show capture view (countdown and capture)
                while not self.control_event.is_set() and self.app_state in [APP_STATE_COUNTDOWN, APP_STATE_CAPTURE, APP_STATE_COMPLETE]:
                    # Show the capture view
                    result = view_capture_screen(
                        self.term, self.data_provider, 
                        self.history, self.state_info, self.control_event
                    )
                    
                    # Process view result
                    if result['action'] == 'cancel':
                        play_sound("Operation canceled")
                        print("[INFO] Operation canceled by user")
                        return False
                    elif result['action'] == 'error':
                        play_sound("Error in capture")
                        print(f"[ERROR] Error in capture view: {result.get('message', 'unknown')}")
                        return False
                    elif result['action'] == 'complete':
                        play_sound("Capture completed")
                        print("[INFO] Capture completed successfully")
                        return True
                        
                    # If we've completed, exit the loop
                    if self.app_state == APP_STATE_COMPLETE:
                        print("[INFO] Complete state detected, ending loop")
                        break
                        
                    time.sleep(0.1)
            
            # Verify final result
            print(f"[INFO] Final state: {self.app_state}")
            if self.app_state == APP_STATE_COMPLETE:
                play_sound("Capture completed")
                return True
            elif self.app_state == APP_STATE_ERROR:
                play_sound("Error in capture")
                return False
            else:
                print("[WARN] Unexpected final state")
                return False
                
        except Exception as e:
            print(f"[ERROR] Error in main controller: {str(e)}")
            return False
        finally:
            # Set control event to stop all threads
            self.control_event.set()
            # Clean up hardware connection
            self.cleanup_hardware()
            print("[INFO] Resources released")



