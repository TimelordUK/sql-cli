#!/bin/bash

# Create docs/images directory if it doesn't exist
mkdir -p docs/images

# Generate timestamp-based filename if none provided
if [ -z "$1" ]; then
    filename="screenshot-$(date +%Y%m%d-%H%M%S).png"
else
    filename="$1"
fi

# Save screenshot from clipboard (WSL2 method)
powershell.exe -c "
\$img = Get-Clipboard -Format Image
if (\$img) {
    \$img.Save('$(wslpath -w $(pwd))/docs/images/$filename')
    Write-Host 'Screenshot saved to docs/images/$filename'
} else {
    Write-Host 'No image in clipboard'
}
"

echo "Screenshot saved to: docs/images/$filename"
echo "Add to README with: ![Description](docs/images/$filename)"