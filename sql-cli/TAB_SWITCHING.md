# F2 Key Mode Switching

## Overview
F2 key provides natural switching between Command and Results modes, keeping Tab free for auto-completion.

## How It Works

### In Results Mode
- **F2** → Switch to Command mode (focus SQL input)
- **Arrows/hjkl** → Navigate data (stay in Results)
- **Esc** → Also switches to Command mode (preserved for compatibility)
- **q** → Quit application

### In Command Mode  
- **F2** → Switch to Results mode (if results exist)
- **Tab** → Auto-complete SQL keywords
- **Enter** → Execute query
- **Arrows** → Navigate command history
- **Text input** → Type SQL queries

## Benefits

1. **No Conflicts**: Tab remains dedicated to auto-completion
2. **Simple**: F2 is easy to remember and press  
3. **No Confusion**: Arrow keys always navigate within current mode
4. **No Accidental Switches**: No boundary-triggered mode changes
5. **Predictable**: F2 always toggles between the two main modes

## Testing

```bash
# Quick test
./test_tab_switch.sh

# Or manually
echo "id,name\n1,Alice\n2,Bob" > test.csv
./target/debug/sql-cli test.csv -e "select * from data"
# Press F2 to switch modes, Tab to complete
```

## Implementation

- Added F2 mapping in `KeyMapper` for both modes
- `SwitchMode` action handles the mode transition
- Validates that results exist before switching to Results mode
- Shows helpful status messages on mode switch

## User Experience

Before:
- Had to use Esc to go from Results to Command
- Tab conflicts with auto-completion
- Arrow keys at boundaries could be confusing

After:
- F2 cleanly toggles between modes
- Tab remains dedicated to auto-completion
- Arrow keys always do what you expect
- No key conflicts