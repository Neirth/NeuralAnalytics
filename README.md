## Neural Analytics - Real-time brain signal analysis with deep learning

![Neural Analytics](https://placehold.co/800x200/ffd433/000000?text=Neural+Analytics&font=raleway)

## Project Description

This project develops a system for real-time analysis of electroencephalographic signals using deep learning techniques, using the BrainBit device to capture occipital and temporal lobe data; LSTM models are used to detect specific patterns and guarantee a response within temporal constraints.

## Key Features

- Acquisition of EEG signals using the BrainBit device.
- Real-time processing with precise temporal constraints.
- Deep learning models based on LSTM architectures.
- Implementation on Raspberry Pi 4 Model B (8GB).
- Compliance with UNE-EN 62304 standards for medical device software.

## Technologies Used

- Hardware:
    - BrainBit device for EEG acquisition.
    - Raspberry Pi 4 Model B (8GB) as processing platform.
    - Tapo Smart Bulb for visual feedback.

- **Software**:
    - BrainFlow SDK for signal acquisition.
    - Deep learning models with LSTM architecture
    - Real-time operating system to guarantee temporal response
    - One-Hot Encoding for signal pre-processing
    - ReLU and Softmax trigger functions

## Requirements

- Raspberry Pi 4 Model B (8GB)
- BrainBit device and Tapo Smart Bulb
- Basic knowledge of real-time systems and deep learning.

## Installation

1. Clone this repository:
     ```
     git clone https://github.dev/Neirth/NeuralAnalytics
     cd NeuralAnalytics
     ```

2. Configure the BrainBit device according to the documentation provided.

3. Run the main application:
     ```
     cargo run --package neural_analytics_gui --release
     ```

4. Enjoy the real-time analysis of EEG signals!

## Project Structure

The project structure is as follows:

```
NeuralAnalytics/
├── .github/                        # GitHub Actions configuration.
├── .vscode/                        # Visual Studio Code configuration.
├── docs/                           # Complete documentation.
├── packages/                       # Source code.
│   ├─── neural_analytics_core/     # Core implementation.
│   ├─── neural_analytics_data/     # Data Capturer.
│   ├─── neural_analytics_gui/      # GUI of Signal acquisition.
│   └─── neural_analytics_model/    # Model building.
├─── LICENSE.md                     # Project license
└─── README.md                      # This file.
```

## Documentation

The complete project documentation is available in the `/docs` folder. It includes:

- Theoretical framework on brain regions and real-time systems.
- Technical specifications of the BrainBit device
- Architecture and evaluation of deep learning models
- Regulatory considerations according to UNE-EN 62304

This documentation is only available in Spanish, and is written in LaTeX format. This format is chosen for its flexibility and the possibility of generating PDF files, required for the final project presentation.

## License

This project is licensed under the GNU General Public License v3.0 - see file [LICENSE.md](LICENSE.md) for details.
