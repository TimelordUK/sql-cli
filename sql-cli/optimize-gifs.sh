#!/bin/bash
# Script to optimize GIF file sizes

echo "Optimizing GIF files..."

# Install gifsicle if not present
if ! command -v gifsicle &> /dev/null; then
    echo "Installing gifsicle..."
    sudo apt-get update && sudo apt-get install -y gifsicle
fi

# Function to optimize a single GIF
optimize_gif() {
    local input="$1"
    local output="${input%.gif}-optimized.gif"
    local original_size=$(du -h "$input" | cut -f1)
    
    echo "Optimizing $input (${original_size})..."
    
    # Aggressive optimization
    gifsicle -O3 --lossy=80 --colors 128 --scale 0.9 "$input" -o "$output"
    
    local new_size=$(du -h "$output" | cut -f1)
    echo "  Created $output (${new_size})"
    
    # Even more aggressive if still too large
    if [[ $(stat -c%s "$output") -gt 3000000 ]]; then
        echo "  Still large, applying more compression..."
        gifsicle -O3 --lossy=120 --colors 64 --scale 0.8 "$input" -o "${input%.gif}-small.gif"
        local tiny_size=$(du -h "${input%.gif}-small.gif" | cut -f1)
        echo "  Created ${input%.gif}-small.gif (${tiny_size})"
    fi
}

# Optimize all GIFs in demos directory
for gif in demos/*.gif; do
    if [[ -f "$gif" && ! "$gif" == *"-optimized.gif" && ! "$gif" == *"-small.gif" ]]; then
        optimize_gif "$gif"
    fi
done

echo ""
echo "Optimization complete!"
echo ""
echo "Tips for smaller GIFs in VHS:"
echo "  - Use smaller dimensions (Width 900-1000, Height 500-600)"
echo "  - Reduce FontSize to 12-14"
echo "  - Use Set PlaybackSpeed 1.5 or 2"
echo "  - Minimize Sleep times"
echo "  - Use Set TypingSpeed to speed up typing"
echo "  - Keep demos under 30 seconds"