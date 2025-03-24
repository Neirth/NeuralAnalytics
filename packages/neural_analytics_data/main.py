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
EEG Capture System with BrainBit
Main entry point for the application.

This script initializes the main controller that manages the entire flow
of EEG data capture, from device configuration to
storage of captured data.

Usage examples:
    # Capture with red scenario (default)
    python main.py 
    
    # Capture with green scenario
    python main.py --type green
    
    # Specify custom MAC address
    python main.py --mac C8:8F:B6:6D:E1:E2
"""

import sys
import traceback
import argparse

from controllers.main_controller import NeuralCaptureController

def main():
    """
    Main function to start the EEG capture application.
    Processes command line arguments and executes the controller.
    """
    print("Starting EEG Capture System with BrainBit...\n")
    
    # Configure parser for command line arguments
    parser = argparse.ArgumentParser(
        prog="Neural Capturer",
        description="EEG Capture System with BrainBit"
    )
    
    # Available arguments
    parser.add_argument('--type', choices=['red', 'green', 'trash'], default='red',
                      help='Type of scenario (red/green)')
    parser.add_argument('--mac',
                      help='MAC address of BrainBit (format: "A0:B1:C2:D3:E4:F5")')
    
    # Parse arguments
    args = parser.parse_args()
    
    try:
        # Create controller with specified parameters
        controller = NeuralCaptureController(
            scenario_type=args.type,
            mac_address=args.mac
        )
        
        # Execute the main capture flow
        print(f"Starting capture with scenario: {args.type.upper()}")
        if args.mac:
            print(f"Using custom MAC address: {args.mac}")
        
        success = controller.run()
        
        # Show result
        if success:
            print("\n[✓] Capture completed successfully.")
            print("    Files have been saved to the data/ directory")
            return 0
        else:
            print("\n[!] The capture was canceled or not completed correctly.")
            return 1
            
    except KeyboardInterrupt:
        print("\n\n[!] Operation canceled by user (Ctrl+C).")
        return 1
    except Exception as e:
        print(f"\n[!] Unexpected error: {str(e)}")
        print("\nError details:")
        traceback.print_exc()
        return 2

if __name__ == "__main__":
    sys.exit(main())