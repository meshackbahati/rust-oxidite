#!/bin/bash

# Script to generate HTML documentation from markdown files using mdbook
# This addresses the user's request to convert documentation to HTML

echo "Setting up mdBook for Oxidite documentation..."

# Install mdbook if not already installed
if ! command -v mdbook &> /dev/null; then
    echo "Installing mdbook..."
    cargo install mdbook
fi

# Navigate to the book directory
cd /home/bealthguy/Public/rust-oxidite/docs/book

echo "Validating book structure..."
mdbook validate

echo "Building HTML documentation..."
mdbook build

echo "Documentation built successfully!"
echo "You can find the HTML output in the 'book-output' directory."
echo ""
echo "To serve the documentation locally, run:"
echo "  cd /home/bealthguy/Public/rust-oxidite/docs/book"
echo "  mdbook serve --open"
echo ""
echo "The documentation is now available as a static website in HTML format."