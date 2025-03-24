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
from sklearn.metrics import accuracy_score

def evaluate_model(model, val_loader, device, writer, epoch=50):
    """
    Evaluates the accuracy of the model on a classification dataset 
    with sliding windows.

    :param model: PyTorch model to evaluate.
    :param val_loader: DataLoader containing the evaluation data.
    :param device: Device where the model runs (CPU or GPU).
    :param writer: TensorBoard writer to log the accuracy.
    :param epoch: Current epoch number, used for TensorBoard logging.
    """
    model.eval()
    all_true = []
    all_pred = []

    with torch.no_grad():
        for element in val_loader:
            # Get inputs and true labels
            x = element['window_features']
            y = element['class']

            # Make predictions
            outputs = model(x)
            # For classification, we use argmax to get the index of the predicted class
            predicted_class = torch.argmax(outputs, dim=1)
            true_class = torch.argmax(y, dim=1)

            all_true.extend(true_class.cpu().numpy())
            all_pred.extend(predicted_class.cpu().numpy())

    accuracy = accuracy_score(all_true, all_pred)
    print(f'[*] Accuracy on the evaluation set: {accuracy:.4f}')

    # Log accuracy in TensorBoard
    writer.add_scalar('Accuracy/eval', accuracy, epoch)