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

import json
import torch
import numpy as np
import pandas as pd
from torch.utils.data import Dataset
from preprocessors.neural_analytics import get_class_label_from_path, onehot_encode_class_label

FEATURE_COLS = ['T3', 'T4', 'O1', 'O2']


class NeuralAnalyticsDataset(Dataset):
    def __init__(self, file_paths: list, window_size: int, device: torch.device, augment: bool = False, stride: int = None, global_stats: dict = None):
        """
        Initializes the dataset for classification. The 'file_paths' is a list of paths to CSV files.

        Each file is processed to generate sliding windows (window_features) and its label
        (class) in one-hot format.
        
        Z-score normalization is applied using GLOBAL FIXED statistics (mean/std) calculated
        from the entire training dataset. This ensures consistency between training and runtime.
        
        :param file_paths: List of paths to CSV files.
        :param window_size: Size of the sliding window.
        :param device: Device to store tensors on.
        :param augment: If True, applies random Gaussian noise to the windows during training.
        :param stride: Step size between windows. If None, defaults to window_size // 2 (50% overlap).
        :param global_stats: Pre-calculated global statistics {'mean': [...], 'std': [...]}. 
                            If None, statistics are calculated from the data.
        """
        self.file_data = []  # List of numpy arrays (N_samples, 4) - RAW data
        self.file_labels = [] # List of one-hot labels
        self.file_paths_valid = []  # Track valid file paths for file-level evaluation
        self.indices = []    # List of (file_idx, start_idx) tuples
        
        self.window_size = window_size
        self.stride = stride if stride is not None else window_size // 2
        self.file_paths = file_paths
        self.augment = augment
        self.device = device
        self.global_stats = global_stats

        # Load all raw data
        all_data_for_stats = []
        skipped_files = 0
        
        for file_idx, file_path in enumerate(self.file_paths):
            try:
                df = pd.read_csv(file_path)
                if not all(col in df.columns for col in FEATURE_COLS):
                    skipped_files += 1
                    continue
                
                signal = df[FEATURE_COLS].values.astype(np.float32)
                
                if len(signal) < self.window_size:
                    skipped_files += 1
                    continue
                    
                label_str = get_class_label_from_path(file_path)
                label_vec = onehot_encode_class_label(label_str)
                
                # Store RAW data
                self.file_data.append(signal)
                self.file_labels.append(label_vec)
                self.file_paths_valid.append(file_path)
                
                # Collect data for global stats calculation
                if self.global_stats is None:
                    all_data_for_stats.append(signal)
                
                # Generate indices
                current_data_idx = len(self.file_data) - 1
                for start_idx in range(0, len(signal) - self.window_size + 1, self.stride):
                    self.indices.append((current_data_idx, start_idx))
                    
            except Exception as e:
                print(f"[!] Error processing {file_path}: {e}")
                skipped_files += 1

        if skipped_files > 0:
            print(f"[!] Skipped {skipped_files} files due to missing columns or insufficient length.")
        
        # Calculate global statistics if not provided
        if self.global_stats is None:
            all_data = np.vstack(all_data_for_stats)
            self.global_stats = {
                'mean': all_data.mean(axis=0).tolist(),
                'std': all_data.std(axis=0).tolist()
            }
            print(f"[*] Calculated global stats from {len(all_data)} samples:")
            print(f"    Mean: {self.global_stats['mean']}")
            print(f"    Std:  {self.global_stats['std']}")
        
        # Convert to numpy arrays for fast normalization
        self.global_mean = np.array(self.global_stats['mean'], dtype=np.float32)
        self.global_std = np.array(self.global_stats['std'], dtype=np.float32)
        self.global_std = np.maximum(self.global_std, 1e-6)  # Avoid division by zero
            
        print(f"[*] Loaded {len(self.file_data)} files, stride={self.stride}, resulting in {len(self.indices)} windows.")
        print(f"[*] Using GLOBAL FIXED z-score normalization")
    
    def get_global_stats(self) -> dict:
        """Returns the global statistics used for normalization."""
        return self.global_stats
    
    def export_normalization_params(self, output_path: str):
        """
        Exports the normalization parameters to a JSON file for use in Rust inference.
        Uses the format expected by the Rust NormalizationParams struct.
        
        :param output_path: Path where to save the JSON file
        """
        params = {
            'global_mean': self.global_stats['mean'],
            'global_std': self.global_stats['std']
        }
        with open(output_path, 'w') as f:
            json.dump(params, f, indent=2)
        print(f"[*] Normalization params exported to: {output_path}")
    
    def _normalize(self, data: np.ndarray) -> np.ndarray:
        """
        Apply GLOBAL z-score normalization using pre-calculated statistics.
        Each channel is normalized using the global mean/std calculated from the entire dataset.
        This ensures consistency between training and runtime inference.
        Formula: (x - global_mean) / global_std for each channel
        """
        # data shape: [timesteps, channels] = [62, 4]
        # global_mean/global_std shape: [4]
        return (data - self.global_mean) / self.global_std
    
    def get_file_count(self):
        """Returns the number of valid files in the dataset."""
        return len(self.file_data)
    
    def get_windows_for_file(self, file_idx: int):
        """
        Returns all windows for a specific file index.
        Useful for file-level evaluation (aggregating predictions per file).
        """
        signal = self.file_data[file_idx]  # Raw data
        label_vec = self.file_labels[file_idx]
        
        windows = []
        for start_idx in range(0, len(signal) - self.window_size + 1, self.stride):
            window_data = signal[start_idx : start_idx + self.window_size]
            # Apply global z-score normalization
            window_data = self._normalize(window_data)
            windows.append(window_data)
        
        if windows:
            windows_array = np.stack(windows).astype(np.float32)
            windows_tensor = torch.tensor(windows_array, dtype=torch.float32).to(self.device)
            label_tensor = torch.tensor(label_vec, dtype=torch.float32).to(self.device)
            return windows_tensor, label_tensor
        return None, None

    def __len__(self):
        return len(self.indices)

    def __getitem__(self, idx):
        file_idx, start_idx = self.indices[idx]
        
        # Retrieve the window from the stored file data (RAW)
        full_signal = self.file_data[file_idx]
        window_data = full_signal[start_idx : start_idx + self.window_size].copy()
        label_vec = self.file_labels[file_idx]
        
        # Apply GLOBAL z-score normalization (same as runtime)
        window_data = self._normalize(window_data)
        
        # Apply data augmentation if enabled (very light)
        if self.augment:
            # Only very light Gaussian noise for regularization
            noise = np.random.normal(0, 0.01, window_data.shape).astype(np.float32)
            window_data = window_data + noise

        window_tensor = torch.tensor(window_data, dtype=torch.float32).to(self.device)
        label_tensor = torch.tensor(label_vec, dtype=torch.float32).to(self.device)
        
        return {
            'window_features': window_tensor,
            'class': label_tensor
        }