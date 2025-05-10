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
import matplotlib.pyplot as plt
from sklearn.metrics import confusion_matrix, ConfusionMatrixDisplay, roc_curve, auc
import torch
from sklearn.metrics import accuracy_score
import numpy as np

from models.neural_analytics import NeuralAnalyticsModel

def evaluate_model(model, val_loader, device, writer, epoch=50, output_dir="build"):
    """
    Evaluates the accuracy of the model on a classification dataset 
    with sliding windows and generates confusion matrix and ROC curve.

    :param model: PyTorch model to evaluate.
    :param val_loader: DataLoader containing the evaluation data.
    :param device: Device where the model runs (CPU or GPU).
    :param writer: TensorBoard writer to log the accuracy.
    :param epoch: Current epoch number, used for TensorBoard logging.
    :param output_dir: Directory where the confusion matrix and ROC curve will be saved.
    :return: Tuple containing validation loss and accuracy.
    """
    model.eval()
    all_true = []
    all_pred = []
    all_scores = []
    total_loss = 0.0
    total_samples = 0

    criterion = torch.nn.CrossEntropyLoss()  # Define the loss function

    with torch.no_grad():
        for element in val_loader:
            # Get inputs and true labels
            x = element['window_features'].to(device)
            y = element['class'].to(device)

            # Make predictions
            outputs = model(x)

            # Calculate loss
            loss = criterion(outputs, torch.argmax(y, dim=1))
            total_loss += loss.item() * x.size(0)
            total_samples += x.size(0)

            # For classification, we use argmax to get the index of the predicted class
            predicted_class = torch.argmax(outputs, dim=1)
            true_class = torch.argmax(y, dim=1)

            all_true.extend(true_class.cpu().numpy())
            all_pred.extend(predicted_class.cpu().numpy())
            all_scores.extend(outputs.cpu().numpy())

    # Calculate average loss and accuracy
    val_loss = total_loss / total_samples
    val_accuracy = accuracy_score(all_true, all_pred)

    print(f'[*] Validation Loss: {val_loss:.4f}, Accuracy: {val_accuracy:.4f}')

    # Log accuracy and loss in TensorBoard
    writer.add_scalar('Loss/Validation', val_loss, epoch)
    writer.add_scalar('Accuracy/Validation', val_accuracy, epoch)

    # Generate and save confusion matrix
    save_confusion_matrix(y_true=all_true, y_pred=all_pred, class_mapping=NeuralAnalyticsModel.class_mapping, output_dir=output_dir)

    # Generate and save ROC curve
    all_true_one_hot = torch.nn.functional.one_hot(torch.tensor(all_true), num_classes=len(set(all_true))).numpy()
    save_roc_curve(y_true=all_true_one_hot, y_scores=np.array(all_scores), class_mapping=NeuralAnalyticsModel.class_mapping, output_dir=output_dir)

    return val_loss, val_accuracy

def save_training_curves(train_losses, train_accuracies, output_dir="build"):
    """
    Saves the training curves (loss and accuracy) in the specified folder.

    :param train_losses: List of training losses per epoch.
    :param train_accuracies: List of training accuracies per epoch.
    :param output_dir: Directory where the plots will be saved.
    """
    os.makedirs(output_dir, exist_ok=True)

    # Loss vs Epochs
    plt.figure()
    plt.plot(train_losses, label='Training')
    plt.title('Loss vs Epochs')
    plt.xlabel('Epochs')
    plt.ylabel('Loss')
    plt.legend()
    plt.savefig(os.path.join(output_dir, 'assets/loss_vs_epochs.png'))
    plt.close()

    # Accuracy vs Epochs
    plt.figure()
    plt.plot(train_accuracies, label='Training')
    plt.title('Accuracy vs Epochs')
    plt.xlabel('Epochs')
    plt.ylabel('Accuracy')
    plt.legend()
    plt.savefig(os.path.join(output_dir, 'assets/accuracy_vs_epochs.png'))
    plt.close()

def save_confusion_matrix(y_true, y_pred, class_mapping, output_dir="build"):
    """
    Saves the confusion matrix in the specified folder.

    :param y_true: True labels.
    :param y_pred: Predicted labels.
    :param class_mapping: Dictionary mapping indices to class names.
    :param output_dir: Directory where the confusion matrix will be saved.
    """
    os.makedirs(output_dir, exist_ok=True)

    cm = confusion_matrix(y_true, y_pred)
    disp = ConfusionMatrixDisplay(confusion_matrix=cm, display_labels=list(class_mapping.values()))
    disp.plot(cmap="Blues", xticks_rotation=45)
    plt.title('Confusion Matrix')
    plt.xlabel('Predicted')
    plt.ylabel('True')
    plt.savefig(os.path.join(output_dir, 'assets/confusion_matrix.png'))
    plt.close()

def save_roc_curve(y_true, y_scores, class_mapping, output_dir="build"):
    """
    Saves the ROC curve and AUC score in the specified folder.

    :param y_true: True labels (one-hot encoded for multilabel).
    :param y_scores: Scores predicted by the model.
    :param class_mapping: Dictionary mapping indices to class names.
    :param output_dir: Directory where the ROC curve will be saved.
    """
    os.makedirs(output_dir, exist_ok=True)

    if y_true.shape[1] > 1:  # Multilabel
        for i, class_name in class_mapping.items():
            fpr, tpr, _ = roc_curve(y_true[:, i], y_scores[:, i])
            roc_auc = auc(fpr, tpr)
            plt.figure()
            plt.plot(fpr, tpr, label=f'{class_name} (AUC = {roc_auc:.2f})')
            plt.plot([0, 1], [0, 1], 'k--')
            plt.title(f'ROC Curve - {class_name}')
            plt.xlabel('False Positive Rate')
            plt.ylabel('True Positive Rate')
            plt.legend()
            plt.savefig(os.path.join(output_dir, f'assets/roc_curve_{class_name}.png'))
            plt.close()
    else:  # Binary
        fpr, tpr, _ = roc_curve(y_true, y_scores)
        roc_auc = auc(fpr, tpr)
        plt.figure()
        plt.plot(fpr, tpr, label=f'AUC = {roc_auc:.2f}')
        plt.plot([0, 1], [0, 1], 'k--')
        plt.title('ROC Curve')
        plt.xlabel('False Positive Rate')
        plt.ylabel('True Positive Rate')
        plt.legend()
        plt.savefig(os.path.join(output_dir, 'assets/roc_curve.png'))
        plt.close()