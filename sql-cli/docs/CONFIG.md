# SQL CLI Configuration Guide

## Configuration File Location

SQL CLI uses a TOML configuration file to customize its behavior and appearance.

### File Locations by Platform

| Platform | Config File Path |
|----------|------------------|
| **Linux/Mac** | `~/.config/sql-cli/config.toml` |
| **Windows** | `%APPDATA%\sql-cli\config.toml` |

### Creating Your Config

1. **Copy the example config:**
   ```bash
   # From the sql-cli directory
   cp config.toml.example ~/.config/sql-cli/config.toml  # Linux/Mac
   ```
   
   **Windows (PowerShell):**
   ```powershell
   # Create directory if it doesn't exist
   mkdir $env:APPDATA\sql-cli
   
   # Copy the example config
   Copy-Item config.toml.example $env:APPDATA\sql-cli\config.toml
   ```

2. **Edit the config:**
   - Linux/Mac: `vim ~/.config/sql-cli/config.toml`
   - Windows: `notepad %APPDATA%\sql-cli\config.toml`

## Configuration Options

### Display Settings

```toml
[display]
use_glyphs = true          # Use Unicode characters for better visuals
show_row_numbers = false   # Show vim-style row numbers
compact_mode = true        # Reduce column padding
```

### Icons (requires `use_glyphs = true`)

```toml
[display.icons]
pin = "📌"
lock = "🔒"
cache = "📦"
file = "📁"
database = "🗄️"
api = "🌐"
```

### Key Bindings

```toml
[keybindings]
vim_mode = true  # Enable vim-style navigation (j/k/h/l)
```

### Behavior

```toml
[behavior]
auto_execute_on_load = true      # Auto-run queries for CSV/JSON files
case_insensitive_default = true  # Default to case-insensitive search
max_display_rows = 10000         # Maximum rows to display
enable_history = true             # Enable command history
max_history_entries = 1000       # History size limit
```

### Theme

```toml
[theme]
color_scheme = "default"          # Color scheme
rainbow_parentheses = true        # Colorful nested parentheses
syntax_highlighting = true        # SQL syntax highlighting
```

## Quick Setup Commands

### Linux/Mac
```bash
# Create config directory and copy default config
mkdir -p ~/.config/sql-cli
cp config.toml.example ~/.config/sql-cli/config.toml
```

### Windows (PowerShell)
```powershell
# Create config directory and copy default config
New-Item -ItemType Directory -Force -Path "$env:APPDATA\sql-cli"
Copy-Item config.toml.example "$env:APPDATA\sql-cli\config.toml"
```

### Windows (Command Prompt)
```cmd
# Create config directory
mkdir "%APPDATA%\sql-cli"
# Then manually copy config.toml.example to that directory
```

## Troubleshooting

### Config Not Loading?

1. **Check the file exists:**
   - Linux/Mac: `ls -la ~/.config/sql-cli/config.toml`
   - Windows: `dir %APPDATA%\sql-cli\config.toml`

2. **Verify TOML syntax:**
   - Make sure all strings are quoted
   - Check for missing commas or brackets
   - Ensure boolean values are `true` or `false` (lowercase)

3. **View debug logs:**
   Run sql-cli with `RUST_LOG=debug` to see config loading messages

### Finding Your Config Directory

- **Linux/Mac:** Run `echo ~/.config/sql-cli/`
- **Windows PowerShell:** Run `echo $env:APPDATA\sql-cli`
- **Windows CMD:** Run `echo %APPDATA%\sql-cli`

## Default Values

If no config file exists, SQL CLI uses these defaults:
- Compact mode: OFF
- Row numbers: OFF  
- Vim mode: ON
- Auto-execute: ON
- Case-insensitive: ON
- Unicode glyphs: ON