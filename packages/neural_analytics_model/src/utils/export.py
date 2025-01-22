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

import torch
import onnx
import os

def export_model(model, device, input_size, output_path):
    """
    Exports a PyTorch model to ONNX format and simplifies the ONNX model.

    :param model: PyTorch model to be exported.
    :param device: PyTorch device being used for training.
    :param input_size: Input size of the model (e.g., (batch_size, channels, height, width)).
    :param output_path: Path where the ONNX model will be saved.
    """
    # Set the model to evaluation mode
    model.eval()

    # Create a dummy input tensor
    dummy_input = torch.randn(*input_size).to(device)

    # Check and create the export directory if it doesn't exist
    output_dir = os.path.dirname(output_path)
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)
        print(f'[*] Directory created: {output_dir}')

    # Export the model to ONNX format
    torch.onnx.export(
        model, dummy_input, output_path,
        export_params=True,
        opset_version=11,
        do_constant_folding=True,  # Constant optimization
        input_names=['input'],
        output_names=['output'],
        dynamic_axes={
            'input': {0: 'batch_size'},  # Dynamic axis for batch size
            'output': {0: 'batch_size'}
        }
    )

    # Load the ONNX model for simplification
    model_onnx = onnx.load(output_path)

    # Save the simplified model
    onnx.save(model_onnx, output_path)

    print(f'[*] Model exported and simplified to: {output_path}')