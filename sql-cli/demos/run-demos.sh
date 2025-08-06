#!/bin/bash
# Script to run all VHS demos

echo "Running VHS demos for sql-cli..."

# Check if VHS is installed
if ! command -v vhs &> /dev/null; then
    echo "VHS not found. Please run ./install-vhs.sh first"
    exit 1
fi

# Create output directory
mkdir -p demos

# Run each demo
demos=(
    "overview"
    "fuzzy-filter"
    "column-navigation"
    "statistics"
    "vim-navigation"
    "sql-queries"
)

for demo in "${demos[@]}"; do
    echo "Generating $demo.gif..."
    vhs "demos/${demo}.tape"
    echo "✓ ${demo}.gif created"
done

echo ""
echo "All demos generated successfully!"
echo "GIF files are in the demos/ directory"
echo ""
echo "To use in README.md:"
echo "![Overview](demos/overview.gif)"