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

def r2_score_torch(y_true, y_pred):
    """
    Calculates R^2 using PyTorch.

    :param y_true: Tensor of true values.
    :param y_pred: Tensor of predicted values.
    :return: R^2 value.
    """
    # Calculate the mean of the true values
    y_true_mean = torch.mean(y_true)

    # Calculate the residual sum of squares and the total sum of squares
    ss_res = torch.sum((y_true - y_pred) ** 2)  # Residual sum of squares
    ss_tot = torch.sum((y_true - y_true_mean) ** 2)  # Total sum of squares

    # Calculate R^2
    r2 = 1 - (ss_res / ss_tot)
    return r2.item()  # Return as a scalar value

def train_model(train_loader, device, writer, epochs=50, learning_rate=0.001):
    """
    Trains the next value prediction model in the electrical grid using sliding windows.

    :param train_loader: DataLoader for the training set.
    :param device: Device (CPU or GPU) to train the model.
    :param writer: TensorBoard writer to log the loss.
    :param epochs: Number of epochs for training.
    :param learning_rate: Learning rate for the optimizer.
    :return: The trained model.
    """
    # Create a model
    model = NeuralAnalyticsModel()
    model.to(device)  # Move the model to the device

    # Define the loss function and optimizer
    criterion = nn.MSELoss()  # Use MSELoss for regression
    optimizer = optim.Adam(model.parameters(), lr=learning_rate)

    start_time = time.time()  # Measure training time

    # Training
    for epoch in range(epochs):
        epoch_loss = 0.0  # To store the accumulated loss in each epoch
        all_outputs = []
        all_targets = []

        for i, element in enumerate(train_loader):
            # Unpack the data
            x = element['window_stack'].to(device)
            y = element['next_value'].to(device)

            # Forward pass
            outputs = model(x)

            # Ensure outputs and y have the same shape
            outputs = torch.squeeze(outputs)

            # Calculate the loss
            loss = criterion(outputs, y)

            # Backward pass and optimization
            optimizer.zero_grad()
            loss.backward()
            optimizer.step()

            epoch_loss += loss.item()

            # Save predicted and true values to calculate R²
            all_outputs.append(outputs.detach())
            all_targets.append(y.detach())

        # Concatenate the lists to get tensors
        all_outputs = torch.cat(all_outputs)
        all_targets = torch.cat(all_targets)

        # Average loss per epoch
        avg_loss = epoch_loss / len(train_loader)

        # Calculate R² using the defined function
        r2 = r2_score_torch(all_targets, all_outputs)

        # Export loss and R² to TensorBoard
        writer.add_scalar('Loss/train', avg_loss, epoch)
        writer.add_scalar('R2/train', r2, epoch)

        # Log each epoch with loss and R²
        print(f'[#] Epoch [{epoch + 1}/{epochs}] -> Loss: {avg_loss:.4f}; R²: {r2:.4f}')

    total_time = time.time() - start_time
    print(f'[*] Training completed in {total_time:.2f} seconds.')

    # Close the SummaryWriter
    writer.close()

    return model