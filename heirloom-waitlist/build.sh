#!/bin/bash

# Build script for Heirloom Waitlist Platform

echo "Building Heirloom Waitlist Platform..."

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "Rust is not installed. Please install Rust first."
    exit 1
fi

# Build the application
cargo build --release

if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "Binary located at: target/release/heirloom-waitlist"
else
    echo "Build failed!"
    exit 1
fi

echo "Build process completed."