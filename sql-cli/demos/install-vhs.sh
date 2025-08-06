#!/bin/bash
# Install VHS for creating terminal GIFs

echo "Installing VHS for terminal recording..."

# For macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    brew install vhs
    echo "VHS installed via Homebrew"
fi

# For Linux
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Check if we have brew
    if command -v brew &> /dev/null; then
        brew install vhs
    else
        # Install via downloading binary
        echo "Installing VHS via curl..."
        # Get the latest release URL
        VHS_VERSION=$(curl -s https://api.github.com/repos/charmbracelet/vhs/releases/latest | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')
        curl -L "https://github.com/charmbracelet/vhs/releases/download/v${VHS_VERSION}/vhs_${VHS_VERSION}_Linux_x86_64.tar.gz" -o vhs.tar.gz
        tar xzf vhs.tar.gz
        # Extract the vhs binary from the versioned directory
        mv vhs_${VHS_VERSION}_Linux_x86_64/vhs ./vhs
        echo "VHS binary extracted. Now installing to /usr/local/bin (requires sudo)..."
        sudo mv vhs /usr/local/bin/
        # Cleanup
        rm -rf vhs.tar.gz vhs_${VHS_VERSION}_Linux_x86_64
    fi
fi

echo "VHS installation complete!"
echo "Test with: vhs --version"