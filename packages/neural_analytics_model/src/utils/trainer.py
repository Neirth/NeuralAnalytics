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

import copy
import time
import torch
import torch.nn as nn
import torch.nn.functional as F
import torch.optim as optim


class FocalLoss(nn.Module):
    """
    Focal Loss for handling class imbalance.
    Reduces the relative loss for well-classified examples, 
    focusing training on hard negatives.
    """
    def __init__(self, gamma=2.0, weight=None, reduction='mean'):
        super().__init__()
        self.gamma = gamma
        self.weight = weight
        self.reduction = reduction
    
    def forward(self, log_probs, targets):
        """
        Args:
            log_probs: Log probabilities from model (after log_softmax or log on softmax output)
            targets: One-hot encoded targets or class indices
        """
        # Convert one-hot to class indices if needed
        if len(targets.shape) > 1 and targets.shape[1] > 1:
            targets = torch.argmax(targets, dim=1)
        
        # Get probabilities from log probs
        probs = torch.exp(log_probs)
        
        # Get the probability for the correct class
        ce_loss = F.nll_loss(log_probs, targets, weight=self.weight, reduction='none')
        
        # Get p_t (probability of correct class)
        p_t = probs.gather(1, targets.unsqueeze(1)).squeeze(1)
        
        # Focal weight: (1 - p_t)^gamma
        focal_weight = (1 - p_t) ** self.gamma
        
        focal_loss = focal_weight * ce_loss
        
        if self.reduction == 'mean':
            return focal_loss.mean()
        elif self.reduction == 'sum':
            return focal_loss.sum()
        return focal_loss


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

def train_model(train_loader, val_loader, device, writer, epochs=100, learning_rate=0.001, class_weights=None):
    """
    Trains the neural analytics classification model with a progress bar at the epoch level.
    Includes validation and saving the best model.

    :param train_loader: DataLoader for the training set.
    :param val_loader: DataLoader for the validation set.
    :param device: Device (CPU or GPU) for training the model.
    :param writer: TensorBoard writer to log metrics.
    :param epochs: Number of epochs for training.
    :param learning_rate: Learning rate for the optimizer.
    :param class_weights: Optional tensor with weights per class for imbalanced datasets.
    :return: The trained model and training metrics (losses and accuracies).
    """
    # Create the model
    model = NeuralAnalyticsModel()
    model.to(device)  # Move model to device

    # Define loss function and optimizer
    # Use Focal Loss with class weights for better handling of hard examples
    if class_weights is not None:
        class_weights = class_weights.to(device)
        criterion = FocalLoss(gamma=2.5, weight=class_weights)
        print(f"[*] Using Focal Loss with gamma=2.5 and class weights: {class_weights.cpu().numpy()}")
    else:
        criterion = FocalLoss(gamma=2.5)
        print(f"[*] Using Focal Loss with gamma=2.5")
    
    optimizer = optim.AdamW(model.parameters(), lr=learning_rate, weight_decay=1e-2)
    scheduler = optim.lr_scheduler.ReduceLROnPlateau(optimizer, mode='min', factor=0.5, patience=15)

    start_time = time.time()  # Measure training time

    # Initialize metrics
    train_losses = []
    train_accuracies = []
    
    best_val_loss = float('inf')
    best_model_state = None

    # Use tqdm for the epoch loop
    with tqdm(range(epochs), desc="[*] Training Progress", unit="epoch") as epoch_bar:
        for epoch in epoch_bar:
            # --- Training Phase ---
            model.train()
            epoch_loss = 0.0
            epoch_accuracy = 0.0
            total_samples = 0

            for batch in train_loader:
                # Unpack data
                x = batch['window_features'].to(device)
                y = batch['class'].to(device)

                # Forward pass
                outputs = model(x)

                # Calculate loss
                loss = criterion(torch.log(outputs + 1e-10), torch.argmax(y, dim=1))

                # Backward pass and optimization
                optimizer.zero_grad()
                loss.backward()
                
                # Gradient clipping
                torch.nn.utils.clip_grad_norm_(model.parameters(), max_norm=1.0)
                
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
            
            # --- Validation Phase ---
            model.eval()
            val_loss = 0.0
            val_accuracy = 0.0
            val_samples = 0
            
            with torch.no_grad():
                for batch in val_loader:
                    x = batch['window_features'].to(device)
                    y = batch['class'].to(device)
                    outputs = model(x)
                    loss = criterion(torch.log(outputs + 1e-10), torch.argmax(y, dim=1))
                    
                    val_loss += loss.item() * x.size(0)
                    val_accuracy += accuracy_torch(outputs, y) * x.size(0)
                    val_samples += x.size(0)
            
            val_loss /= val_samples
            val_accuracy /= val_samples

            # Save best model
            if val_loss < best_val_loss:
                best_val_loss = val_loss
                best_model_state = copy.deepcopy(model.state_dict())

            # Get current learning rate
            current_lr = optimizer.param_groups[0]['lr']
            
            # Apply the scheduler based on validation loss
            scheduler.step(val_loss)

            # Log metrics
            train_losses.append(epoch_loss)
            train_accuracies.append(epoch_accuracy)
            writer.add_scalar('Loss/Train', epoch_loss, epoch)
            writer.add_scalar('Accuracy/Train', epoch_accuracy, epoch)
            writer.add_scalar('Loss/Validation', val_loss, epoch)
            writer.add_scalar('Accuracy/Validation', val_accuracy, epoch)
            writer.add_scalar('Learning_Rate', current_lr, epoch)

            # Update tqdm description
            epoch_bar.set_postfix(
                loss=f"{epoch_loss:.3f}", 
                acc=f"{epoch_accuracy:.3f}", 
                val_loss=f"{val_loss:.3f}", 
                val_acc=f"{val_accuracy:.3f}",
                lr=f"{current_lr:.1e}"
            )

    # Restore best model
    if best_model_state is not None:
        print(f"[*] Restoring best model with Validation Loss: {best_val_loss:.4f}")
        model.load_state_dict(best_model_state)

    # Get final learning rate
    final_lr = optimizer.param_groups[0]['lr']
    
    print(f"[*] Training completed in {time.time() - start_time:.2f} seconds.")

    return model, train_losses, train_accuracies