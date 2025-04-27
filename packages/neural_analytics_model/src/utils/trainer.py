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

from models.neural_analytics import NeuralAnalyticsModel

import time
import torch
import torch.nn as nn
import torch.optim as optim

def accuracy_torch(outputs, targets):
    """
    Calculates accuracy using PyTorch.
    
    :param outputs: Tensor with model predictions (probabilities)
    :param targets: Tensor with true labels (in one-hot format)
    :return: Accuracy value
    """
    # Get the indices of predicted classes
    _, predicted = torch.max(outputs, dim=1)
    
    # Convert targets from one-hot to class indices
    if len(targets.shape) > 1 and targets.shape[1] > 1:  # if one-hot
        targets = torch.argmax(targets, dim=1)
    
    # Calculate accuracy
    correct = (predicted == targets).sum().item()
    total = targets.size(0)
    
    return correct / total

def train_model(train_loader, device, writer, epochs=200, learning_rate=0.001):
    """
    Trains the neural analytics classification model.
    
    :param train_loader: DataLoader for the training set.
    :param device: Device (CPU or GPU) for training the model.
    :param writer: TensorBoard writer to log metrics.
    :param epochs: Number of epochs for training.
    :param learning_rate: Learning rate for the optimizer.
    :return: The trained model.
    """
    # Create the model
    model = NeuralAnalyticsModel()
    model.to(device)  # Move model to device

    # Define loss function and optimizer
    criterion = nn.CrossEntropyLoss()  # Use CrossEntropyLoss for classification
    optimizer = optim.Adam(model.parameters(), lr=learning_rate)

    start_time = time.time()  # Measure training time

    # Training
    for epoch in range(epochs):
        model.train()
        epoch_loss = 0.0  # To store cumulative loss in each epoch
        epoch_accuracy = 0.0  # To store cumulative accuracy in each epoch
        
        for batch in train_loader:
            # Unpack data
            x = batch['window_features'].to(device)
            y = batch['class'].to(device)
            
            # Forward pass
            outputs = model(x)
            
            # Calculate loss
            loss = criterion(outputs, torch.argmax(y, dim=1))
            
            # Backward pass and optimization
            optimizer.zero_grad()
            loss.backward()
            optimizer.step()
            
            epoch_loss += loss.item()
            epoch_accuracy += accuracy_torch(outputs, y)
        
        # Epoch averages
        avg_loss = epoch_loss / len(train_loader)
        avg_accuracy = epoch_accuracy / len(train_loader)
        
        # Export metrics to TensorBoard
        writer.add_scalar('Loss/train', avg_loss, epoch)
        writer.add_scalar('Accuracy/train', avg_accuracy, epoch)
        
        # Log each epoch with loss and accuracy
        print(f'[#] Epoch [{epoch+1}/{epochs}] -> Loss: {avg_loss:.4f}; Accuracy: {avg_accuracy:.4f}')
    
    total_time = time.time() - start_time
    print(f'[*] Training completed in {total_time:.2f} seconds.')
    
    return model