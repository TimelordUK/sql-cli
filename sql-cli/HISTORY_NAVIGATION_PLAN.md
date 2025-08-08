# History Navigation Enhancement Plan

## Current Behavior
- **Up Arrow in Results**: Switch to Command mode (saves position)
- **Down Arrow in Command**: Switch to Results mode (if results exist)
- **Ctrl+R**: Opens mcfly-style fuzzy history search
- **Multi-line mode**: Up/Down navigate within text

## Proposed Enhancement
Add traditional command-line history navigation (like bash/zsh) while preserving current muscle memory.

### Option 1: Context-Aware Navigation (Recommended)
**Behavior:**
- **Command Mode + No Results**: Up/Down navigate history
- **Command Mode + Has Results**: Current behavior (Up stays in command, Down goes to results)
- **Results Mode**: Current behavior (Up goes to command)
- **Ctrl+L or ESC+C**: Clear results view (enables history navigation again)
- **Ctrl+P/Ctrl+N**: Always navigate history (alternative keybinding)

**Clear Results Flow:**
1. User has query results displayed
2. User presses Ctrl+L (or types `:clear`)
3. Results are cleared, view resets
4. Now Up/Down arrows navigate history
5. Fresh start for new query exploration

### Option 2: Configurable Behavior
Add a config option: `history_navigation_mode`
- `"classic"`: Up/Down always navigate history in Command mode
- `"smart"`: Context-aware (Option 1)
- `"modal"`: Current behavior (mode switching)

### Option 3: Modifier Key
- **Up/Down**: Current behavior (mode switching)
- **Alt+Up/Alt+Down**: Navigate history
- **Ctrl+P/Ctrl+N**: Navigate history (Emacs-style)

## Implementation Plan

### Phase A: Add History to InputManager
1. Add history tracking to InputManager trait:
   ```rust
   trait InputManager {
       // ... existing methods ...
       fn set_history(&mut self, history: Vec<String>);
       fn history_previous(&mut self) -> bool;
       fn history_next(&mut self) -> bool;
       fn get_history_index(&self) -> Option<usize>;
   }
   ```

2. Track history position and temp buffer:
   - Current input (what user is typing)
   - History index (which history item is showing)
   - Temp storage (save current input when navigating)

### Phase B: Integrate with Buffer
1. Add history state to Buffer:
   ```rust
   pub struct Buffer {
       // ... existing fields ...
       input_history_index: Option<usize>,
       input_temp_storage: Option<String>, // Store current input when navigating
   }
   ```

2. Add BufferAPI methods:
   ```rust
   fn navigate_history_up(&mut self) -> bool;
   fn navigate_history_down(&mut self) -> bool;
   fn reset_history_position(&mut self);
   ```

### Phase C: Update Key Handling
1. Modify Up/Down handling in Command mode:
   ```rust
   KeyCode::Up => {
       if self.get_results().is_none() {
           // No results, navigate history
           buffer.navigate_history_up();
       } else {
           // Has results, current behavior
           self.set_mode(AppMode::Results);
       }
   }
   ```

2. Add Ctrl+P/Ctrl+N as dedicated history keys

### Phase D: History Integration
1. Connect to existing CommandHistory
2. Filter history by context (current table/schema if available)
3. Smart sorting (frequency, recency, relevance)

## Benefits
1. **Familiar UX**: Matches standard CLI behavior
2. **Preserves Muscle Memory**: Existing users won't be disrupted
3. **Enhanced Productivity**: Quick access to recent queries
4. **Context-Aware**: Smart behavior based on current state

## Testing Plan
1. Test history navigation with no results
2. Test mode switching with results
3. Test history preservation when switching modes
4. Test multi-line mode (should not affect)
5. Test Ctrl+P/N alternative bindings
6. Test history filtering/relevance

## Configuration
Add to config.toml:
```toml
[input]
# History navigation behavior
# Options: "smart" (default), "classic", "modal"
history_navigation = "smart"

# Maximum history items to keep in memory for quick access
history_buffer_size = 100

# Filter history by current context (table/schema)
context_aware_history = true
```

## Migration Notes
- This enhancement fits naturally into our InputManager migration
- Can be implemented incrementally without breaking existing behavior
- Provides immediate value to users while we complete the migration