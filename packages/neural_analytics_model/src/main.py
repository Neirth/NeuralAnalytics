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

from utils.trainer import train_model
from utils.export import export_model
from utils.evaluation import evaluate_model

from datasets.neural_analytics import NeuralAnalyticsDataset
from models.neural_analytics import NeuralAnalyticsModel
from sklearn.model_selection import train_test_split

from torch.utils.data import DataLoader
from torch.utils.tensorboard import SummaryWriter

import os
import torch

BATCH_SIZE = 64
DATA_FILE = os.path.join(os.getcwd(), './assets/AUSTRIA_2015_2021.csv')

def main():
    # Notify about the intention of this module
    print(f'[*] Training module for the {NeuralAnalyticsModel.__name__} model')

    # We try to use the best device as possible
    device = torch.device('cuda' if torch.cuda.is_available() else 'mps' if torch.backends.mps.is_available() else 'cpu')
    torch.set_default_dtype(torch.float32)

    print(f'[*] The device to be used will be "{device}"')

    # Prepare dataset
    dataset = NeuralAnalyticsDataset(DATA_FILE, device)
    train_dataset, val_dataset = train_test_split(dataset, test_size=0.2, random_state=42)

    # Load the dataset into PyTorch
    train_loader = DataLoader(train_dataset, batch_size=BATCH_SIZE, shuffle=True)
    val_loader = DataLoader(val_dataset, batch_size=BATCH_SIZE, shuffle=False)  # No need to shuffle during validation

    # Prepare to capture data in TensorBoard
    writer = SummaryWriter(log_dir="./runs")

    # Train and Test Model
    model = train_model(train_loader, device, writer)
    evaluate_model(model, val_loader, device, writer)

    # Export Model
    export_model(
        model,
        device,
        input_size=(1, 19, 3),
        output_path='./build/neural_analytics.onnx'
    )

    # Close the training report
    writer.close()

if __name__ == "__main__":
    main()