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
import torch
import numpy as np
from torch.utils.data import Dataset
from preprocessors.neural_analytics import neural_analytics_preprocessor

class NeuralAnalyticsDataset(Dataset):
    def __init__(self, dataset_folder: str, window_size: int, device: torch.device):
        """
        Initializes the dataset for classification. The 'dataset_folder' is expected to
        contain subfolders representing classes (for example, red, green, or trash), and each
        subfolder should contain CSV files with the data.
        
        Each file is processed to generate sliding windows (window_features) and its label
        (class) in one-hot format. All windows from all files are combined into the dataset.
        """
        self.window_features = []
        self.labels = []

        # Recursively traverse the dataset folder
        for root, dirs, files in os.walk(dataset_folder):
            for file in files:
                if file.endswith('.csv'):
                    file_path = os.path.join(root, file)
                    # We preprocess the CSV file to obtain its sliding windows.
                    # The neural_analytics_preprocessor function extracts the label from the path.
                    df_windows = neural_analytics_preprocessor(file_path, window_size)
                    # The obtained windows and labels are added to the global dataset.
                    self.window_features.extend(df_windows['window_features'].tolist())
                    self.labels.extend(df_windows['class'].tolist())

        # Convert the list of windows and labels to numpy arrays.
        self.window_features = np.array(self.window_features, dtype=np.float32)
        self.labels = np.array(self.labels, dtype=np.float32)
        self.device = device

    def __len__(self):
        return len(self.labels)

    def __getitem__(self, idx):
        window_tensor = torch.tensor(self.window_features[idx], dtype=torch.float32).to(self.device)
        label_tensor = torch.tensor(self.labels[idx], dtype=torch.float32).to(self.device)
        return {
            'window_features': window_tensor,
            'class': label_tensor
        }