#!/usr/bin/env bash
set -euo pipefail

# Configuration
SCRIPT_DIR="$(pwd)"
INSTALL_DIR="${HOME}/bin"
BINARY_NAME="codex"
CARGO_PROJECT_DIR="codex-rs"
SOUND_FILE="${SCRIPT_DIR}/assets/sounds/evil-laugh.mp3"

echo "Codex Local Installation Script"
echo "================================="

# Function to play sound if available
play_sound() {
    if command -v mpg123 &> /dev/null && [ -f "$SOUND_FILE" ]; then
        mpg123 "$SOUND_FILE" &> /dev/null &
    fi
}

# Check if we're in the right directory
if [ ! -d "$CARGO_PROJECT_DIR" ]; then
    echo "Error: $CARGO_PROJECT_DIR directory not found!"
    echo "Please run this script from the root of the codex repository."
    exit 1
fi

# Create ~/bin if it doesn't exist
if [ ! -d "$INSTALL_DIR" ]; then
    echo "Creating $INSTALL_DIR directory..."
    mkdir -p "$INSTALL_DIR"
fi

# Build the release binary
echo "Building release binary..."
cd "$CARGO_PROJECT_DIR"
cargo build --release --bin "$BINARY_NAME"

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

# Copy the binary to ~/bin
echo "Installing binary to $INSTALL_DIR..."
cp "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Check if ~/bin is in PATH
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo "Warning: $INSTALL_DIR is not in your PATH!"
    echo ""
    echo "To add it to your PATH, add this line to your shell configuration file:"
    echo "  export PATH=\"\$HOME/bin:\$PATH\""
    echo ""
fi

# Verify installation
if [ -f "$INSTALL_DIR/$BINARY_NAME" ]; then
    echo "✓ Installation successful!"
    play_sound
    echo ""
    echo "Binary installed to: $INSTALL_DIR/$BINARY_NAME"

    # Get version if possible
    if command -v "$INSTALL_DIR/$BINARY_NAME" &> /dev/null; then
        VERSION=$("$INSTALL_DIR/$BINARY_NAME" --version 2>/dev/null || echo "unknown")
        echo "Version: $VERSION"
    fi

    if echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo ""
        echo "You can now run: codex"
    else
        echo ""
        echo "After adding $INSTALL_DIR to your PATH, you can run: codex"
    fi
else
    echo "✗ Installation failed!"
    exit 1
fi