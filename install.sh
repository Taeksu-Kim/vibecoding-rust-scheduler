#!/bin/bash

# Scheduler Installation Script
# Supports macOS and Linux

set -e

echo "🗓️  Installing Scheduler..."
echo ""

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed."
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "✓ Rust detected: $(rustc --version)"

# Build release
echo ""
echo "📦 Building release binary..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✓ Build successful"

# Determine install location
if [ "$(uname)" == "Darwin" ]; then
    INSTALL_DIR="$HOME/.local/bin"
elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
    INSTALL_DIR="$HOME/.local/bin"
else
    echo "❌ Unsupported platform: $(uname)"
    exit 1
fi

# Create install directory if it doesn't exist
mkdir -p "$INSTALL_DIR"

# Copy binary
echo ""
echo "📥 Installing to $INSTALL_DIR..."
cp target/release/scheduler "$INSTALL_DIR/sched"
chmod +x "$INSTALL_DIR/sched"

echo "✓ Binary installed as 'sched'"

# Check if install dir is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "⚠️  $INSTALL_DIR is not in your PATH"
    echo ""
    echo "Add this line to your ~/.bashrc or ~/.zshrc:"
    echo ""
    echo "    export PATH=\"\$PATH:$INSTALL_DIR\""
    echo ""
else
    echo "✓ Installation directory is in PATH"
fi

# Create config directory
CONFIG_DIR="$HOME/.config/scheduler"
mkdir -p "$CONFIG_DIR"
echo "✓ Config directory created at $CONFIG_DIR"

# Create data directory
DATA_DIR="$HOME/.local/share/scheduler/data"
mkdir -p "$DATA_DIR"
mkdir -p "$DATA_DIR/history"
echo "✓ Data directory created at $DATA_DIR"

echo ""
echo "✅ Installation complete!"
echo ""
echo "Quick start:"
echo "  sched add \"My Task\" --start 09:00 --end 10:00"
echo "  sched start"
echo "  sched ui"
echo ""
echo "For help:"
echo "  sched --help"
echo ""
