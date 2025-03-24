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
import torch.nn as nn

INPUT_SIZE = 4  # Number of features in the input (T3, T4, O1, O2)
HIDDEN_SIZE = 64  # Size of hidden units in the LSTM
NUM_CLASSES = 3  # Number of classification categories (RED, GREEN, UNKNOWN)

class NeuralAnalyticsModel(nn.Module):
    def __init__(self):
        super(NeuralAnalyticsModel, self).__init__()

        # Define individual components
        self.lstm = nn.LSTM(INPUT_SIZE, HIDDEN_SIZE, 4, batch_first=True, bidirectional=False)
        
        # Linear components in Sequential with ReLU and additional layer
        self.model = nn.Sequential(
            nn.Linear(HIDDEN_SIZE, HIDDEN_SIZE // 2),  # First dense layer reduces dimensionality
            nn.ReLU(),                                 # ReLU activation for non-linearity
            nn.Linear(HIDDEN_SIZE // 2, NUM_CLASSES),  # Second dense layer for classification
            nn.Softmax(dim=1)                          # Softmax for probabilities
        )
    
        # Mapping from index to class label
        self.class_mapping = {0: 'RED', 1: 'GREEN', 2: 'UNKNOWN'}

    def forward(self, x, initial_states=None):
        # x shape: (batch_size, seq_length, input_size)
        
        # Process through LSTM with optional initial states
        if initial_states is not None:
            lstm_out, _ = self.lstm(x, initial_states)
        else:
            lstm_out, _ = self.lstm(x)  # PyTorch inicializará estados automáticamente
        
        # Take the output from the last time step
        last_out = lstm_out[:, -1, :]
        
        # Pass through the sequential model (Linear + ReLU + Linear + Softmax)
        probabilities = self.model(last_out)
        
        return probabilities
