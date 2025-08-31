# Action System Debug Tools

We've created two powerful debug tools to visualize and understand the action system in real-time:

## 1. action_logger - Simple Console Logger

A lightweight tool that prints key mappings to the console as you type.

### Usage
```bash
cargo run --bin action_logger
```

### Features
- Shows each key press and what action it maps to
- Displays vim-style count accumulation (e.g., "5j" → Navigate(Down(5)))
- Simple console output, easy to understand
- Minimal dependencies

### Example Output
```
│ j      │ => Navigate(Down(1))
│ k      │ => Navigate(Up(1))
│ 5      │ Building count: 5 │
│ j      │ Count: 5   │ => Navigate(Down(5))
│ v      │ => ToggleSelectionMode
│ p      │ => ToggleColumnPin
│ s      │ => Sort(None)
```

## 2. action_debugger - Full TUI Debugger

A complete TUI application for exploring the action system interactively.

### Usage
```bash
cargo run --bin action_debugger
```

### Features
- **Status Panel**: Shows current mode, selection mode, and count buffer
- **Action History**: Lists recent key-to-action mappings (newest first)
- **Key History**: Shows raw key presses
- **State Tracking**: Updates mode and selection state based on actions
- **Visual Feedback**: Color-coded output for easy reading

### Interface Layout
```
┌─ Status ─────────────────────────────┐
│ Mode: Results   Selection: Row       │
│ Count Buffer: (none)                 │
│ Try: j/k, 5j, v, p, s, F1, q        │
└──────────────────────────────────────┘

┌─ Action History ────────────────────┐
│ Key 'j' → Navigate(Down(1))        │
│ Count '5' + Key 'j' → Navigate...  │
│ Key 'v' → ToggleSelectionMode      │
│ ...                                 │
└─────────────────────────────────────┘

┌─ Key History ───────────────────────┐
│ j                                   │
│ 5                                   │
│ v                                   │
└─────────────────────────────────────┘
```

## Quick Demo

Run the demo script to choose which tool to try:

```bash
./demo_action_tools.sh
```

## Key Combinations to Try

### Basic Navigation
- `j`, `k`, `h`, `l` - Vim-style movement
- Arrow keys - Standard navigation
- `PageUp`, `PageDown` - Page navigation
- `Home`, `End` - Jump to start/end

### Vim-Style Counts
- `5j` - Move down 5 rows
- `10k` - Move up 10 rows
- `3l` - Move right 3 columns

### Mode & UI
- `v` - Toggle selection mode (Row/Cell/Column)
- `F1` - Show help
- `F5` - Show debug info
- `Esc` - Exit current mode

### Data Operations
- `p` - Toggle column pin
- `s` - Sort by current column
- `q` - Quit application

## Understanding the Output

### Action Types
- `Navigate(Down(1))` - Move cursor down 1 row
- `Navigate(Up(5))` - Move cursor up 5 rows (from vim count)
- `ToggleSelectionMode` - Switch between Row/Cell/Column selection
- `Sort(None)` - Sort by current column (None means use current)
- `ToggleColumnPin` - Pin/unpin current column

### Count Buffer
When you type a number, it goes into the count buffer. The next navigation command will use this count:
1. Type `5` → Count buffer shows "5"
2. Type `j` → Action becomes `Navigate(Down(5))`
3. Count buffer clears

## Why These Tools?

1. **Development**: Understand how keys map to actions while developing
2. **Debugging**: See if a key is being mapped correctly
3. **Documentation**: Generate examples of key mappings
4. **Testing**: Verify the action system is working as expected
5. **Learning**: Understand the vim-style count system

## Implementation Notes

These tools directly use the same `KeyMapper` that the main SQL CLI uses, so they show exactly what would happen in the real application. They're invaluable for:
- Adding new key mappings
- Debugging why a key isn't working
- Understanding the vim-style count system
- Testing the action system in isolation