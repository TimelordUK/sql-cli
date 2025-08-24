# Shadow State Manager - Current Status

## What It Does

The shadow state manager runs alongside the existing state system, observing and logging all state transitions without affecting behavior. It's our "learning system" that will help us understand the actual state flow before centralizing state management.

## Current Observation Points

### Complete List of Observed Transitions:

1. **Mode Switches (SwitchModeWithCursor action)** - Handles 'a', 'A', 'i', 'I' keys
2. **Escape Key Transitions** - Returns to Command or Results from various modes
3. **Query Execution** - Transition to Results after query success
4. **Vim Search** - Start/end of vim search mode
5. **Column Search** - Column search mode transitions
6. **Help Mode** - F1 or ? key for help
7. **History Mode** - Ctrl+R for command history
8. **Jump to Row** - : key for row navigation
9. **Column Stats** - Statistical view of columns
10. **Filter Application** - Return to Results after applying filters

### Key Observation Points:

- **Command ↔ Results**: Via 'a'/'i' keys, Escape, query execution
- **Results → Search**: Via '/' for vim search, column search
- **Search → Results**: Via Escape or filter application
- **Any → Help**: Via F1 or ?
- **Any → History**: Via Ctrl+R
- **Results → JumpToRow**: Via ':'

## Where to See It

### In the TUI:

1. **Status Line** - Always shows current shadow state (now in Cyan for better visibility):
   - `[Shadow: COMMAND]` - In command/query mode
   - `[Shadow: RESULTS]` - Viewing results
   - `[Shadow: SEARCH(Vim)]` - In vim search mode
   - `[Shadow: SEARCH(Column)]` - In column search mode
   - `[Shadow: HELP]` - In help mode
   - `[Shadow: HISTORY]` - In history search
   - `[Shadow: JUMP_TO_ROW]` - In jump to row mode
   - `[Shadow: COLUMN_STATS]` - Viewing column statistics

2. **Debug View (F5)** - Shows full shadow state info:
   - Current state
   - Recent transitions (last 5)
   - Total transition count
   - Any discrepancies detected

### In Logs:

Run with: `RUST_LOG=shadow_state=info ./target/release/sql-cli file.csv`

You'll see:
```
[INFO shadow_state] Shadow state manager initialized
[INFO shadow_state] [#1] COMMAND -> RESULTS (trigger: execute_query_success)
[INFO shadow_state] [#2] RESULTS -> SEARCH(Vim) (trigger: slash_key_pressed)
[INFO shadow_state] [#3] Exiting search -> RESULTS (trigger: search_cancelled)
```

## What's Been Added

✅ **All major mode transitions now observed:**
- SwitchModeWithCursor action (a, A, i, I keys)
- Escape key transitions (all modes)
- Column search mode
- Help mode (F1 or ?)
- History mode (Ctrl+R)
- Jump to row mode (:)
- Column statistics view
- Filter application returns

## What's Still Missing

We still need to add observations for:
- [ ] Tab completion mode
- [ ] Some edge case transitions
- [ ] Mode-specific sub-states (e.g., different filter types)

## Next Steps

1. Run the app and watch the shadow state in status line
2. Press F5 to see debug info including shadow state history
3. Add more observation points as we identify patterns
4. Use the logs to understand actual vs expected transitions

## The Goal

Once we understand all the state transitions through observation, we can:
1. Design proper state management that matches actual behavior
2. Identify missing side effects
3. Find redundant or conflicting state changes
4. Build a centralized state manager that works correctly

The shadow state is our "training wheels" - it helps us learn the system before we change it!