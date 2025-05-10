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
from tqdm import tqdm  # Import tqdm for progress bars

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

def train_model(train_loader, device, writer, epochs=1000, learning_rate=0.001):
    """
    Trains the neural analytics classification model with a progress bar at the epoch level.

    :param train_loader: DataLoader for the training set.
    :param device: Device (CPU or GPU) for training the model.
    :param writer: TensorBoard writer to log metrics.
    :param epochs: Number of epochs for training.
    :param learning_rate: Learning rate for the optimizer.
    :return: The trained model and training metrics (losses and accuracies).
    """
    # Create the model
    model = NeuralAnalyticsModel()
    model.to(device)  # Move model to device

    # Define loss function and optimizer
    
    criterion = nn.CrossEntropyLoss()  # Use CrossEntropyLoss for classification
    optimizer = optim.Adam(model.parameters(), lr=learning_rate)
    scheduler = optim.lr_scheduler.ReduceLROnPlateau(optimizer, mode='min', factor=0.5, patience=10)

    start_time = time.time()  # Measure training time

    # Initialize metrics
    train_losses = []
    train_accuracies = []

    # Use tqdm for the epoch loop
    with tqdm(range(epochs), desc="[*] Training Progress", unit="epoch") as epoch_bar:
        for epoch in epoch_bar:
            model.train()
            epoch_loss = 0.0  # To store cumulative loss in each epoch
            epoch_accuracy = 0.0  # To store cumulative accuracy in each epoch
            total_samples = 0

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

                # Update metrics
                batch_loss = loss.item() * x.size(0)
                batch_accuracy = accuracy_torch(outputs, y) * x.size(0)
                epoch_loss += batch_loss
                epoch_accuracy += batch_accuracy
                total_samples += x.size(0)

            # Calculate average loss and accuracy for the epoch
            epoch_loss /= total_samples
            epoch_accuracy /= total_samples

            # Get current learning rate
            current_lr = optimizer.param_groups[0]['lr']
            
            # Apply the scheduler
            scheduler.step(epoch_loss)

            # Log metrics
            train_losses.append(epoch_loss)
            train_accuracies.append(epoch_accuracy)
            writer.add_scalar('Loss/Train', epoch_loss, epoch)
            writer.add_scalar('Accuracy/Train', epoch_accuracy, epoch)
            writer.add_scalar('Learning_Rate', current_lr, epoch)

            # Update tqdm description with epoch metrics including learning rate
            epoch_bar.set_postfix(loss=epoch_loss, accuracy=epoch_accuracy, lr=current_lr)

    # Get final learning rate
    final_lr = optimizer.param_groups[0]['lr']
    
    print(f"[*] Training Loss: {epoch_loss:.4f}, Training Accuracy: {epoch_accuracy:.4f}, Learning Rate: {final_lr:.6f}")
    print(f"[*] Training completed in {time.time() - start_time:.2f} seconds.")

    return model, train_losses, train_accuracies