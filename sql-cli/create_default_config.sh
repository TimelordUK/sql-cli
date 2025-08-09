#!/bin/bash

# Script to create default config.toml for sql-cli

cat > config.toml.example << 'EOF'
# SQL CLI Configuration File
# Copy this to the appropriate location:
# - Linux/Mac: ~/.config/sql-cli/config.toml
# - Windows: %APPDATA%\sql-cli\config.toml

[display]
use_glyphs = true
show_row_numbers = false
compact_mode = true

[display.icons]
pin = "📌"
lock = "🔒"
cache = "📦"
file = "📁"
database = "🗄️"
api = "🌐"
case_insensitive = "Ⓘ"
warning = "⚠️"
error = "❌"
info = "ℹ️"
success = "✅"

[keybindings]
vim_mode = true

[behavior]
auto_execute_on_load = true
case_insensitive_default = true
max_display_rows = 10000
enable_history = true
max_history_entries = 1000

[theme]
color_scheme = "default"
rainbow_parentheses = true
syntax_highlighting = true
EOF

echo "Default config created as config.toml.example"
echo ""
echo "Copy to the appropriate location:"
echo "  Linux/Mac: cp config.toml.example ~/.config/sql-cli/config.toml"
echo "  Windows (in PowerShell): Copy-Item config.toml.example \$env:APPDATA\\sql-cli\\config.toml"