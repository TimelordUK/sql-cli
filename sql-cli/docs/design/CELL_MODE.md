# Cell Mode vs Row Mode

SQL CLI now supports two selection modes, similar to csvlens, optimized for different workflows.

## Quick Start

Press `v` to toggle between Cell and Row mode.

## Cell Mode

**Purpose**: Focus on individual cell values, perfect for copying IDs or specific values to other systems.

### Visual Indicators
- Only the selected cell is highlighted
- Status line shows `[CELL]` mode indicator
- Current cell value displayed in status line
- Cell highlighted with yellow text, bold and underlined (by default)

### Key Behaviors
- `y` - Instantly yanks the current cell value
- Navigation moves cell by cell
- Clear visual focus on exactly what will be copied

### Status Line Example
```
[NAV] [CELL] Row 45/1000 | Col: platformOrderId = TRD-2024-0042 | Yanked: dealId=DEAL-789
```

## Row Mode (Default)

**Purpose**: Traditional row-based selection for working with entire records.

### Visual Indicators
- Entire row is highlighted with dark gray background
- Status line shows `[ROW]` mode indicator
- Row selection arrow `►` shows current row

### Key Behaviors
- `y` - Enters yank mode (yy for row, yc for column, ya for all)
- Navigation moves row by row
- Column highlighting shows current field

## Customizing Cell Highlighting

The cell highlighting style can be customized in your config file to work better with your terminal color scheme.

### Default Style (Yellow Foreground)
```toml
[theme.cell_selection_style]
foreground = "yellow"
use_background = false
bold = true
underline = true
```

### Alternative Styles

#### Orange/Amber for Gruvbox
```toml
[theme.cell_selection_style]
foreground = "yellow"  # Often renders as orange in Gruvbox
use_background = false
bold = true
underline = false  # Less visual noise
```

#### High Contrast
```toml
[theme.cell_selection_style]
foreground = "black"
use_background = true
background = "yellow"
bold = true
underline = false
```

#### Subtle Style
```toml
[theme.cell_selection_style]
foreground = "cyan"
use_background = false
bold = false
underline = true
```

#### Minimal
```toml
[theme.cell_selection_style]
foreground = "white"
use_background = false
bold = true
underline = false
```

## Color Options

Available colors for `foreground` and `background`:
- `"black"`
- `"red"`
- `"green"`
- `"yellow"` (often orange in many themes)
- `"blue"`
- `"magenta"`
- `"cyan"`
- `"white"`
- `"gray"` / `"grey"`
- `"dark_gray"` / `"dark_grey"`

## Use Cases

### Cell Mode - Copying IDs
1. Load trades data
2. Press `v` to enter cell mode
3. Navigate to the `tradeId` column
4. Press `y` to copy just that ID
5. Paste into another application

### Row Mode - Analyzing Records
1. Load data (default row mode)
2. Navigate through records
3. Press `yy` to copy entire row
4. Or press `yc` to copy entire column

## Tips

1. **Quick ID Copy**: In cell mode, `y` immediately copies - no need for `yy`
2. **Visual Clarity**: The highlighting makes it crystal clear what will be yanked
3. **Status Feedback**: Always shows what was last yanked
4. **Terminal Compatibility**: If colors are hard to read, customize the style in your config

## Troubleshooting

### Cell Highlighting Hard to Read?

If the default yellow text is hard to read on your color scheme:

1. Generate a config file if you don't have one:
   ```bash
   sql-cli --generate-config
   ```

2. Edit `~/.config/sql-cli/config.toml` (Linux/Mac) or `%APPDATA%\sql-cli\config.toml` (Windows)

3. Try different color combinations:
   - For dark themes: `foreground = "cyan"` or `foreground = "magenta"`
   - For light themes: `foreground = "blue"` or `foreground = "red"`
   - Disable underline if it's too busy: `underline = false`

4. Restart SQL CLI to apply changes