# Voice Cloning Script
# Uses your 3 WAV files to create a voice clone

param(
    [Parameter(Mandatory=$true)]
    [string]$VoiceName,
    
    [Parameter(Mandatory=$true)]
    [string[]]$AudioFiles,
    
    [switch]$GPU
)

Write-Host "🎤 Voice Cloning Script" -ForegroundColor Green
Write-Host "Voice Name: $VoiceName" -ForegroundColor Cyan
Write-Host "Audio Files: $($AudioFiles -join ', ')" -ForegroundColor Cyan

# Validate audio files
foreach ($file in $AudioFiles) {
    if (-not (Test-Path $file)) {
        Write-Error "Audio file not found: $file"
        exit 1
    }
    
    if (-not $file.EndsWith(".wav")) {
        Write-Error "Only WAV files are supported: $file"
        exit 1
    }
}

# Create voice models directory
$voiceModelsDir = "voice_models"
if (-not (Test-Path $voiceModelsDir)) {
    New-Item -ItemType Directory -Path $voiceModelsDir
}

# Create voice-specific directory
$voiceDir = Join-Path $voiceModelsDir $VoiceName
if (-not (Test-Path $voiceDir)) {
    New-Item -ItemType Directory -Path $voiceDir
}

Write-Host "📁 Voice directory: $voiceDir" -ForegroundColor Yellow

# Copy audio files to voice directory
foreach ($file in $AudioFiles) {
    $fileName = Split-Path $file -Leaf
    $destPath = Join-Path $voiceDir $fileName
    Copy-Item $file $destPath -Force
    Write-Host "✅ Copied: $fileName" -ForegroundColor Green
}

# Create voice configuration
$voiceConfig = @{
    name = $VoiceName
    audio_files = $AudioFiles
    created_at = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
    gpu_enabled = $GPU
    sample_rate = 16000
    channels = 1
}

$configPath = Join-Path $voiceDir "voice_config.json"
$voiceConfig | ConvertTo-Json -Depth 10 | Out-File $configPath -Encoding UTF8

Write-Host "✅ Voice configuration saved to: $configPath" -ForegroundColor Green

# Create training script for the voice
$trainingScript = @"
# Voice Training Script for $VoiceName

Write-Host "🎤 Training voice model for: $VoiceName" -ForegroundColor Green

# Set up environment
`$env:CUDA_VISIBLE_DEVICES = "0"
`$env:PYTHONPATH = "."

# Install required packages
pip install torch torchaudio transformers datasets accelerate

# Run voice training
python train_voice.py `
    --voice_name "$VoiceName" `
    --audio_dir "$voiceDir" `
    --output_dir "$voiceDir/models" `
    --epochs 1000 `
    --batch_size 4 `
    --learning_rate 0.0001 `
    --gpu_enabled `$$($GPU.IsPresent)

Write-Host "✅ Voice training completed!" -ForegroundColor Green
"@

$trainingScriptPath = Join-Path $voiceDir "train_voice.ps1"
$trainingScript | Out-File $trainingScriptPath -Encoding UTF8

Write-Host "✅ Training script created: $trainingScriptPath" -ForegroundColor Green

# Create Python training script
$pythonScript = @"
import torch
import torchaudio
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader
import os
import json
import argparse
from pathlib import Path

class VoiceDataset(Dataset):
    def __init__(self, audio_dir, sample_rate=16000):
        self.audio_dir = Path(audio_dir)
        self.sample_rate = sample_rate
        self.audio_files = list(self.audio_dir.glob("*.wav"))
        
    def __len__(self):
        return len(self.audio_files)
    
    def __getitem__(self, idx):
        audio_file = self.audio_files[idx]
        waveform, sample_rate = torchaudio.load(audio_file)
        
        # Resample if needed
        if sample_rate != self.sample_rate:
            resampler = torchaudio.transforms.Resample(sample_rate, self.sample_rate)
            waveform = resampler(waveform)
        
        # Convert to mono if stereo
        if waveform.shape[0] > 1:
            waveform = torch.mean(waveform, dim=0, keepdim=True)
        
        return waveform.squeeze()

class VoiceModel(nn.Module):
    def __init__(self, input_size=16000, hidden_size=512, num_layers=3):
        super(VoiceModel, self).__init__()
        self.lstm = nn.LSTM(input_size, hidden_size, num_layers, batch_first=True)
        self.fc = nn.Linear(hidden_size, input_size)
        
    def forward(self, x):
        lstm_out, _ = self.lstm(x)
        output = self.fc(lstm_out)
        return output

def train_voice(args):
    print(f"🎤 Training voice model for: {args.voice_name}")
    
    # Load dataset
    dataset = VoiceDataset(args.audio_dir)
    dataloader = DataLoader(dataset, batch_size=args.batch_size, shuffle=True)
    
    # Initialize model
    device = torch.device("cuda" if torch.cuda.is_available() and args.gpu_enabled else "cpu")
    model = VoiceModel().to(device)
    
    # Loss and optimizer
    criterion = nn.MSELoss()
    optimizer = optim.Adam(model.parameters(), lr=args.learning_rate)
    
    # Training loop
    model.train()
    for epoch in range(args.epochs):
        total_loss = 0
        for batch_idx, audio in enumerate(dataloader):
            audio = audio.to(device)
            
            # Forward pass
            optimizer.zero_grad()
            output = model(audio)
            loss = criterion(output, audio)
            
            # Backward pass
            loss.backward()
            optimizer.step()
            
            total_loss += loss.item()
            
            if batch_idx % 10 == 0:
                print(f"Epoch {epoch+1}/{args.epochs}, Batch {batch_idx}, Loss: {loss.item():.4f}")
        
        avg_loss = total_loss / len(dataloader)
        print(f"Epoch {epoch+1}/{args.epochs}, Average Loss: {avg_loss:.4f}")
        
        # Save checkpoint
        if (epoch + 1) % 100 == 0:
            checkpoint_path = os.path.join(args.output_dir, f"checkpoint_epoch_{epoch+1}.pth")
            torch.save(model.state_dict(), checkpoint_path)
            print(f"✅ Checkpoint saved: {checkpoint_path}")
    
    # Save final model
    final_model_path = os.path.join(args.output_dir, "voice_model.pth")
    torch.save(model.state_dict(), final_model_path)
    print(f"✅ Final model saved: {final_model_path}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Voice Training Script")
    parser.add_argument("--voice_name", required=True, help="Name of the voice")
    parser.add_argument("--audio_dir", required=True, help="Directory containing WAV files")
    parser.add_argument("--output_dir", required=True, help="Output directory for models")
    parser.add_argument("--epochs", type=int, default=1000, help="Number of training epochs")
    parser.add_argument("--batch_size", type=int, default=4, help="Batch size")
    parser.add_argument("--learning_rate", type=float, default=0.0001, help="Learning rate")
    parser.add_argument("--gpu_enabled", action="store_true", help="Enable GPU training")
    
    args = parser.parse_args()
    train_voice(args)
"@

$pythonScriptPath = Join-Path $voiceDir "train_voice.py"
$pythonScript | Out-File $pythonScriptPath -Encoding UTF8

Write-Host "✅ Python training script created: $pythonScriptPath" -ForegroundColor Green

# Create voice synthesis script
$synthesisScript = @"
# Voice Synthesis Script for $VoiceName

param(
    [Parameter(Mandatory=`$true)]
    [string]`$Text,
    
    [string]`$OutputFile = "output.wav",
    
    [switch]`$GPU
)

Write-Host "🔊 Synthesizing speech for: $VoiceName" -ForegroundColor Green
Write-Host "Text: `$Text" -ForegroundColor Cyan

# Load voice model
`$modelPath = Join-Path "$voiceDir" "models/voice_model.pth"
if (-not (Test-Path `$modelPath)) {
    Write-Error "Voice model not found. Please train the voice first: .\train_voice.ps1"
    exit 1
}

# Run synthesis
python synthesize_voice.py `
    --text "`$Text" `
    --model_path "`$modelPath" `
    --output_file "`$OutputFile" `
    --gpu_enabled `$$(`$GPU.IsPresent)

Write-Host "✅ Speech synthesized: `$OutputFile" -ForegroundColor Green
"@

$synthesisScriptPath = Join-Path $voiceDir "synthesize_voice.ps1"
$synthesisScript | Out-File $synthesisScriptPath -Encoding UTF8

Write-Host "✅ Synthesis script created: $synthesisScriptPath" -ForegroundColor Green

Write-Host "🎤 Voice cloning setup completed!" -ForegroundColor Green
Write-Host "📁 Voice files in: $voiceDir" -ForegroundColor Cyan
Write-Host "🚀 To train your voice: .\train_voice.ps1" -ForegroundColor Yellow
Write-Host "🔊 To synthesize speech: .\synthesize_voice.ps1 -Text 'Hello world'" -ForegroundColor Yellow 