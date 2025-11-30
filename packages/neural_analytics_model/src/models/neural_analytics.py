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
import torch.nn as nn
import torch.nn.functional as F

INPUT_SIZE = 4       # Number of features in the input (T3, T4, O1, O2)
NUM_CLASSES = 3      # Number of classification categories (RED, GREEN, TRASH)
DROPOUT = 0.3        # Moderate dropout to allow learning subtle patterns

# CNN-LSTM Hybrid Architecture
# CNN extracts local spatial/frequency patterns from EEG signals
# LSTM captures temporal dependencies over the extracted features
# This is the standard approach for EEG classification in literature

class NeuralAnalyticsModel(nn.Module):
    # Mapping from index to class label
    class_mapping = {0: 'RED', 1: 'GREEN', 2: 'TRASH'}

    def __init__(self):
        super(NeuralAnalyticsModel, self).__init__()

        # ============== CNN Feature Extractor ==============
        # Input: (batch, seq_len, 4) -> permute to (batch, 4, seq_len) for Conv1d
        # NOTE: Input data is expected to be z-score normalized per-channel per-window
        # This is done in preprocessing to ensure consistency between training and inference
        
        # First Conv Block: extract low-level patterns (edges, peaks)
        self.conv1 = nn.Conv1d(
            in_channels=INPUT_SIZE,
            out_channels=16,  # Reduced from 32
            kernel_size=5,
            padding=2  # Same padding
        )
        self.bn1 = nn.BatchNorm1d(16)
        self.drop1 = nn.Dropout(DROPOUT)  # Stronger dropout
        self.pool1 = nn.MaxPool1d(kernel_size=2, stride=2)  # Reduce seq_len by half
        
        # Second Conv Block: extract higher-level patterns
        self.conv2 = nn.Conv1d(
            in_channels=16,
            out_channels=32,  # Reduced from 64
            kernel_size=3,
            padding=1  # Same padding
        )
        self.bn2 = nn.BatchNorm1d(32)
        self.drop2 = nn.Dropout(DROPOUT)  # Stronger dropout
        self.pool2 = nn.MaxPool1d(kernel_size=2, stride=2)  # Reduce seq_len by half again
        
        # ============== LSTM Temporal Encoder ==============
        # After 2 pooling layers: seq_len / 4
        # Input to LSTM: (batch, seq_len/4, 32)
        self.lstm = nn.LSTM(
            input_size=32,  # Reduced
            hidden_size=32,  # Reduced from 64
            num_layers=1,
            batch_first=True,
            bidirectional=True,
            dropout=0.0  # No dropout for single layer
        )
        
        # ============== Classifier ==============
        # LSTM output: 32 * 2 (bidirectional) = 64
        self.classifier = nn.Sequential(
            nn.Linear(64, 32),  # Reduced
            nn.ReLU(),
            nn.Dropout(DROPOUT),
            nn.Linear(32, NUM_CLASSES),
            nn.Softmax(dim=1)
        )

    def forward(self, x, initial_states=None):
        # x shape: (batch_size, seq_length, input_size) e.g., (B, 62, 4)
        # Input is expected to be z-score normalized per-channel
        
        # ============== CNN Feature Extraction ==============
        # Permute for Conv1d: (batch, channels, seq_len)
        x = x.permute(0, 2, 1)  # (B, 4, 62)
        
        # Conv Block 1 (data is already normalized)
        x = self.conv1(x)       # (B, 16, 62)
        x = self.bn1(x)
        x = F.relu(x)
        x = self.drop1(x)       # Regularization
        x = self.pool1(x)       # (B, 16, 31)
        
        # Conv Block 2
        x = self.conv2(x)       # (B, 32, 31)
        x = self.bn2(x)
        x = F.relu(x)
        x = self.drop2(x)       # Regularization
        x = self.pool2(x)       # (B, 32, 15)
        
        # ============== LSTM Temporal Encoding ==============
        # Permute back for LSTM: (batch, seq_len, features)
        x = x.permute(0, 2, 1)  # (B, 15, 32)
        
        # Process through LSTM
        if initial_states is not None:
            lstm_out, _ = self.lstm(x, initial_states)
        else:
            lstm_out, _ = self.lstm(x)
        
        # lstm_out: (B, 15, 64) - bidirectional doubles hidden size
        
        # Use only the last hidden state (captures full sequence context)
        last_hidden = lstm_out[:, -1, :]  # (B, 64)
        
        # ============== Classification ==============
        probabilities = self.classifier(last_hidden)
        
        return probabilities
