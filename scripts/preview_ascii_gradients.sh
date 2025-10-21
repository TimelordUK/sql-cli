#!/bin/bash
# Preview different color gradients for ASCII_ART('sql-cli')

set -e

# Build if needed
if [ ! -f target/release/sql-cli ]; then
    echo "Building sql-cli..."
    cargo build --release
fi

CLI="./target/release/sql-cli"

echo "=== ASCII Art Gradient Previews ==="
echo ""

echo "1. Cyan-to-Blue (professional/tech):"
$CLI -q "WITH lines AS (SELECT line, ROW_NUMBER() OVER () as n, COUNT(*) OVER () as total FROM ASCII_ART('sql-cli')) SELECT ANSI_RGB(0, ROUND(255 - (255 * (n - 1) / (total - 1))), 255, line) FROM lines" -o tsv
echo ""
echo ""

echo "2. Magenta-to-Cyan (vibrant, modern):"
$CLI -q "WITH lines AS (SELECT line, ROW_NUMBER() OVER () as n, COUNT(*) OVER () as total FROM ASCII_ART('sql-cli')) SELECT ANSI_RGB(ROUND(255 - (255 * (n - 1) / (total - 1))), ROUND(0 + (255 * (n - 1) / (total - 1))), 255, line) FROM lines" -o tsv
echo ""
echo ""

echo "3. Fire (red-to-yellow):"
$CLI -q "WITH lines AS (SELECT line, ROW_NUMBER() OVER () as n, COUNT(*) OVER () as total FROM ASCII_ART('sql-cli')) SELECT ANSI_RGB(255, ROUND(0 + (255 * (n - 1) / (total - 1))), 0, line) FROM lines" -o tsv
echo ""
echo ""

echo "4. Ocean (deep blue to cyan):"
$CLI -q "WITH lines AS (SELECT line, ROW_NUMBER() OVER () as n, COUNT(*) OVER () as total FROM ASCII_ART('sql-cli')) SELECT ANSI_RGB(0, ROUND(128 + (127 * (n - 1) / (total - 1))), ROUND(200 + (55 * (n - 1) / (total - 1))), line) FROM lines" -o tsv
echo ""
echo ""

echo "5. Purple gradient (original):"
$CLI -q "WITH lines AS (SELECT * FROM ASCII_ART('sql-cli')) SELECT ANSI_RGB(148, 0, 211, line) FROM lines" -o tsv
echo ""
echo ""

echo "6. Green Matrix (hacker aesthetic):"
$CLI -q "WITH lines AS (SELECT line, ROW_NUMBER() OVER () as n, COUNT(*) OVER () as total FROM ASCII_ART('sql-cli')) SELECT ANSI_RGB(0, ROUND(150 + (105 * (n - 1) / (total - 1))), 0, line) FROM lines" -o tsv
echo ""
