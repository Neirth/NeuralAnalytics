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
import pandas as pd
import numpy as np
from sklearn.preprocessing import MinMaxScaler

def normalize_features(df: pd.DataFrame, features: list) -> pd.DataFrame:
    """
    Globally normalizes the specified columns using MinMaxScaler.
    """
    scaler = MinMaxScaler()
    df_norm = df.copy()
    df_norm[features] = scaler.fit_transform(df_norm[features])
    # print("[*] Feature normalization completed for:", features)
    return df_norm

def get_class_label_from_path(file: str) -> str:
    """
    Gets the class label from the folder path.
    Looks for 'red', 'green' or 'trash' in the path (in lowercase).
    """
    file_lower = file.lower()
    if "red" in file_lower:
        return "red"
    elif "green" in file_lower:
        return "green"
    elif "trash" in file_lower:
        return "trash"
    else:
        return "unknown"

def onehot_encode_class_label(class_label: str) -> np.ndarray:
    """
    Converts the class label to a one-hot vector.
    Mapping:
      - 'red'   -> [1, 0, 0]
      - 'green' -> [0, 1, 0]
      - 'trash' -> [0, 0, 1]
    """
    mapping = {
        "red": [1, 0, 0],
        "green": [0, 1, 0],
        "trash": [0, 0, 1],
    }
    return np.array(mapping.get(class_label, [0, 0]))

def create_present_sliding_windows(df: pd.DataFrame, window_size: int, class_label: str) -> pd.DataFrame:
    """
    Creates sliding windows from the dataset.

    Each window is a consecutive sequence (of size window_size) of columns T3, T4, O1 and O2.
    Each window is assigned the label (one-hot encoded) obtained from the folder path.
    """
    feature_cols = ['T3', 'T4', 'O1', 'O2']
    df = normalize_features(df, feature_cols)
    
    windows = []
    for i in range(len(df) - window_size + 1):
        window_features = df.loc[i:i + window_size - 1, feature_cols].values
        encoded_label = onehot_encode_class_label(class_label)
        windows.append((window_features, encoded_label))

    if not windows:
        print("[!] No windows were generated, check the dataset size and window parameter.")
    
    window_df = pd.DataFrame(windows, columns=['window_features', 'class'])
    # print("[*] Sliding windows successfully created. Last entries:")
    # print(window_df.tail())

    return window_df

def neural_analytics_preprocessor(file: str, window_size: int) -> pd.DataFrame:
    """
    Preprocesses the current dataset for classification.

    The CSV is expected to contain at least the columns:
      - 'T3', 'T4', 'O1', 'O2': Numerical features to be used by the model.
    
    The class label is extracted from the folder path and one-hot encoding is applied.
    Then, sliding windows of size window_size are generated and the label is assigned to each window.
    """
    # Read the CSV and filter the required columns.
    df = pd.read_csv(file, on_bad_lines='skip', delimiter=",", low_memory=False)
    required_cols = ['T3', 'T4', 'O1', 'O2']
    df = df.dropna(subset=required_cols)
    # print("[*] View of preprocessed dataset:")
    # print(df.head())
    df = df[required_cols].copy()
    
    # Get the class label from the file path.
    class_label = get_class_label_from_path(file)
    # print("[*] Class obtained from path:", class_label)
    
    # Generate sliding windows with the assigned label.
    window_df = create_present_sliding_windows(df, window_size, class_label)
    
    return window_df