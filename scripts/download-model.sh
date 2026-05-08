#!/bin/bash
# Download a GGUF model for Local Assistant

set -e

MODELS_DIR="models"
mkdir -p "$MODELS_DIR"

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║         Local Assistant - Model Downloader                  ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

echo "Available models:"
echo ""
echo "1) Mistral 7B Instruct (Q4_K_M) - 4.3GB - Recommended"
echo "   Balanced quality/speed, good for general use"
echo ""
echo "2) Phi-3 Mini (Q4) - 2.3GB - Fast & Small"
echo "   Great for limited RAM/older Macs"
echo ""
echo "3) Gemma 2B (Q4_K_M) - 1.6GB - Tiny"
echo "   Minimal resource usage"
echo ""
read -p "Select model (1-3) or press Enter for default (1): " choice

case "$choice" in
    2|"")
        MODEL_NAME="phi-3-mini-q4"
        MODEL_URL="https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf"
        FILENAME="phi-3-mini-q4.gguf"
        ;;
    3)
        MODEL_NAME="gemma-2b-q4"
        MODEL_URL="https://huggingface.co/lmstudio-community/Gemma-2B-GGUF/resolve/main/gemma-2b-q4_k_m.gguf"
        FILENAME="gemma-2b-q4.gguf"
        ;;
    *)
        MODEL_NAME="mistral-7b-instruct-q4"
        MODEL_URL="https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf"
        FILENAME="mistral-7b-instruct-q4.gguf"
        ;;
esac

OUTPUT="$MODELS_DIR/$FILENAME"

if [ -f "$OUTPUT" ]; then
    echo "✓ Model already exists: $OUTPUT"
    read -p "Download again? (y/N): " retry
    if [[ ! "$retry" =~ ^[Yy]$ ]]; then
        exit 0
    fi
    rm "$OUTPUT"
fi

echo ""
echo "Downloading: $MODEL_NAME"
echo "URL: $MODEL_URL"
echo "Destination: $OUTPUT"
echo ""

# Check for curl or wget
if command -v curl &> /dev/null; then
    curl -L --progress-bar "$MODEL_URL" -o "$OUTPUT"
elif command -v wget &> /dev/null; then
    wget --show-progress "$MODEL_URL" -O "$OUTPUT"
else
    echo "Error: Neither curl nor wget found. Please install one."
    exit 1
fi

echo ""
echo "✓ Download complete!"
echo "Model saved to: $OUTPUT"
du -h "$OUTPUT"
