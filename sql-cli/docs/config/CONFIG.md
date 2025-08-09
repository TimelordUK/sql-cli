# SQL CLI Configuration Guide

SQL CLI supports extensive configuration through a TOML configuration file. This allows you to customize the appearance, behavior, and keybindings to match your preferences.

## Quick Start

### Initialize Configuration with Wizard
```bash
sql-cli --init-config
```
This interactive wizard will ask you a few questions and create a personalized configuration.

### Generate Default Configuration File
```bash
sql-cli --generate-config
```
This creates a fully commented configuration file with all default values that you can edit.

## Configuration File Location

- **Linux/macOS**: `~/.config/sql-cli/config.toml`
- **Windows**: `%APPDATA%\sql-cli\config.toml`

## Configuration Options

### Display Settings

```toml
[display]
# Use Unicode/Nerd Font glyphs for icons
# Set to false for ASCII-only mode (better compatibility for terminals without font support)
use_glyphs = true

# Show row numbers by default in results view
show_row_numbers = false

# Use compact mode by default (less padding, more data visible)
compact_mode = false
```

### Icon Customization

When `use_glyphs = true`, you can customize the icons used throughout the interface:

```toml
[display.icons]
pin = "📌"              # Pinned columns
lock = "🔒"             # Viewport lock
cache = "📦"            # Cache indicator
file = "📁"             # File data source
database = "🗄️"        # Database source
api = "🌐"              # API data source
case_insensitive = "Ⓘ"  # Case-insensitive mode
warning = "⚠️"          # Parser warnings
error = "❌"            # Errors
info = "ℹ️"             # Information
success = "✅"          # Success indicators
```

When `use_glyphs = false`, all icons automatically switch to ASCII alternatives:
- `[P]` for pin
- `[L]` for lock
- `[C]` for cache
- `[F]` for file
- etc.

### Keybinding Settings

```toml
[keybindings]
# Use vim-style keybindings (j/k navigation, yy to yank, etc.)
vim_mode = true

# Custom key mappings (future feature)
# [keybindings.custom_mappings]
# "copy_row" = "ctrl+c"
# "paste" = "ctrl+v"
```

### Behavior Settings

```toml
[behavior]
# Automatically execute SELECT * when loading CSV/JSON files
auto_execute_on_load = true

# Use case-insensitive string comparisons by default
case_insensitive_default = false

# Maximum rows to display without warning
max_display_rows = 10000

# Cache directory (leave commented to use default)
# cache_dir = "/path/to/cache"

# Enable query history
enable_history = true

# Maximum number of history entries to keep
max_history_entries = 1000
```

### Theme Settings

```toml
[theme]
# Color scheme: "default", "dark", "light", "solarized"
color_scheme = "default"

# Enable rainbow parentheses in SQL queries
rainbow_parentheses = true

# Enable syntax highlighting
syntax_highlighting = true
```

## Simple Mode for Recording/Demos

If you're recording GIFs or demos, or using a terminal that doesn't support Unicode/Nerd Fonts, enable simple mode:

```toml
[display]
use_glyphs = false  # This will use ASCII characters instead of icons
```

This replaces all Unicode glyphs with simple ASCII alternatives:
- `📌` → `[P]` (pin)
- `🔒` → `[L]` (lock)
- `📦` → `[C]` (cache)
- etc.

## Vim-Like Keybindings

When `vim_mode = true`, the following keybindings are available:

### Navigation (Results Mode)
- `j`/`k` - Move down/up
- `h`/`l` - Move left/right  
- `g`/`G` - Go to first/last row
- `0`/`$` - Go to first/last column

### Clipboard Operations
- `y` - Enter yank mode
  - `yy` - Yank current row (tab-separated)
  - `yc` - Yank current column (all values)
  - `ya` - Yank all data (CSV format, respects filters)

## Example Configurations

### Minimal ASCII Configuration
```toml
[display]
use_glyphs = false
compact_mode = true

[behavior]
auto_execute_on_load = false
```

### Power User Configuration
```toml
[display]
use_glyphs = true
show_row_numbers = true
compact_mode = false

[keybindings]
vim_mode = true

[behavior]
auto_execute_on_load = true
max_history_entries = 5000

[theme]
rainbow_parentheses = true
syntax_highlighting = true
```

## Troubleshooting

### Icons Not Displaying Correctly
If you see boxes or question marks instead of icons:
1. Run `sql-cli --init-config` and answer "n" when asked about Unicode support
2. Or manually edit your config and set `use_glyphs = false`

### Configuration Not Loading
- Check the file exists at the correct location
- Ensure the TOML syntax is valid (no syntax errors)
- Look for error messages when starting sql-cli

### Resetting to Defaults
Delete the configuration file and run `sql-cli --generate-config` to create a fresh one with all defaults.