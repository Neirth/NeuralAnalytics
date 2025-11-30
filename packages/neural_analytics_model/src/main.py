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

from utils.trainer import train_model
from utils.export import export_model
from utils.evaluation import evaluate_model, save_training_curves, evaluate_model_file_level

from datasets.neural_analytics import NeuralAnalyticsDataset
from models.neural_analytics import NeuralAnalyticsModel
from sklearn.model_selection import train_test_split

from torch.utils.data import DataLoader
from torch.utils.tensorboard import SummaryWriter
from collections import Counter

import os
import random
import numpy as np
from pathlib import Path
import torch

# Reproducibility
SEED = 42
random.seed(SEED)
np.random.seed(SEED)
torch.manual_seed(SEED)

BATCH_SIZE = 64      # Standard batch size
WINDOW_SIZE = 62
WINDOW_STRIDE = 31   # 50% overlap
REPO_ROOT = Path(__file__).resolve().parents[3]
DATASET_FOLDER = REPO_ROOT / "dataset"
BUILD_DIR = REPO_ROOT / "build"
ASSETS_DIR = BUILD_DIR / "assets"
RUNS_DIR = BUILD_DIR / "runs"

def main():
    # Notify about the purpose of this module
    print(f'[*] Training module for the {NeuralAnalyticsModel.__name__} model')

    # Select the best available device
    device = torch.device('cuda' if torch.cuda.is_available() else 'mps' if torch.backends.mps.is_available() else 'cpu')
    torch.set_default_dtype(torch.float32)
    print(f'[*] The device to be used will be "{device}"')

    # Prepare the dataset from the folder with class subfolders
    train_files = []
    val_files = []
    
    # Iterate over each class directory to ensure stratified split
    for class_dir in sorted([d for d in DATASET_FOLDER.iterdir() if d.is_dir()]):
        files = sorted([str(f) for f in class_dir.glob("*.csv")])
        if not files:
            continue
        
        class_name = class_dir.name.upper()
        
        # Stratified split with fixed seed and shuffle for better distribution
        t_files, v_files = train_test_split(
            files, 
            test_size=0.2, 
            shuffle=True,      # Shuffle to avoid temporal bias
            random_state=SEED  # Fixed seed for reproducibility
        )
        
        print(f"    {class_name}: {len(t_files)} train, {len(v_files)} val")
        train_files.extend(t_files)
        val_files.extend(v_files)
    
    print(f"[*] Total: {len(train_files)} train, {len(val_files)} val")
    
    # Verify class balance in validation set
    val_classes = [Path(f).parent.name for f in val_files]
    print(f"[*] Val distribution: {dict(Counter(val_classes))}")

    # Create datasets with controlled stride (50% overlap instead of 98%+ overlap)
    # Both use z-score normalization per window (same as runtime)
    # First create train_dataset to compute global normalization statistics
    train_dataset = NeuralAnalyticsDataset(train_files, WINDOW_SIZE, device, augment=False, stride=WINDOW_STRIDE)
    
    # Use the same global_stats from training for validation to ensure consistent normalization
    train_global_stats = train_dataset.get_global_stats()
    val_dataset = NeuralAnalyticsDataset(val_files, WINDOW_SIZE, device, augment=False, stride=WINDOW_STRIDE, global_stats=train_global_stats)

    # Load the dataset in PyTorch
    train_loader = DataLoader(train_dataset, batch_size=BATCH_SIZE, shuffle=True)
    val_loader = DataLoader(val_dataset, batch_size=BATCH_SIZE, shuffle=False)  # No shuffle in validation

    # Create folders for saving the model and training logs
    BUILD_DIR.mkdir(parents=True, exist_ok=True)
    ASSETS_DIR.mkdir(parents=True, exist_ok=True)
    RUNS_DIR.mkdir(parents=True, exist_ok=True)

    # Configure TensorBoard
    writer = SummaryWriter(log_dir=str(RUNS_DIR))

    # Train and evaluate the model
    epochs = int(os.getenv("TRAIN_EPOCHS", "200"))
    learning_rate = float(os.getenv("TRAIN_LR", "0.0005"))
    
    # Class weights to balance accuracy across classes
    # Order: [RED=0, GREEN=1, TRASH=2]
    # GREEN has highest confusion (11 misclassified as RED), so increase its weight
    # TRASH is already very accurate, can reduce slightly
    class_weights = torch.tensor([1.0, 1.3, 0.9], dtype=torch.float32)

    model, train_losses, train_accuracies = train_model(
        train_loader,
        val_loader,
        device,
        writer,
        epochs=epochs,
        learning_rate=learning_rate,
        class_weights=class_weights,
    )
    
    # Window-level evaluation
    val_losses, val_accuracies = evaluate_model(
        model, 
        val_loader, 
        device, writer,
        output_dir=str(BUILD_DIR)
    )
    
    # File-level evaluation (aggregate predictions per file)
    print("\n[*] File-level evaluation (aggregating predictions per file):")
    file_accuracy = evaluate_model_file_level(model, val_dataset, device)
    print(f"[*] File-level accuracy: {file_accuracy:.2%}")

    # Export training curves
    save_training_curves(
        train_losses=train_losses,
        train_accuracies=train_accuracies,
        output_dir=str(BUILD_DIR)
    )

    # Export the model
    export_model(
        model,
        device,
        input_size=(1, WINDOW_SIZE, 4),
        output_path=str(BUILD_DIR / 'neural_analytics.onnx')
    )
    
    # Export normalization parameters for Rust inference
    train_dataset.export_normalization_params(str(BUILD_DIR / 'normalization_params.json'))
    print(f"[*] Normalization params exported to {BUILD_DIR / 'normalization_params.json'}")

    # Close the training log
    writer.close()

if __name__ == "__main__":
    main()